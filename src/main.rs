mod api;
mod auth;
mod conf;
mod consts;
mod db;
mod errors;
mod front;
mod matomo;
mod models;
mod stores;
#[cfg(test)]
mod tests;
mod thingsup;
mod types;
mod utils;

use std::collections::HashMap;
use std::fs;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use auth::backend::Backend;
use axum::routing::Router;
use axum::middleware;
use axum_login::AuthManagerLayerBuilder;
use chrono::{DateTime, Utc};
use clap::Parser;
use miette::{IntoDiagnostic, miette};
use owo_colors::OwoColorize;
use tokio::net::{TcpListener, UnixListener};
use tokio::signal;
use tokio::sync::mpsc;
use tokio::time;
use tower_http::trace::TraceLayer;
use tower_sessions::{Expiry, SessionManagerLayer};
use tracing::info;

use consts::{MATOMO_SITE_ID, MATOMO_URL};
use thingsup::{AppOptions, Commands, config_jinja, config_logging, get_binding_addr};
use matomo::AIChatbotEvent;
use types::{AppState, BindingAddr};

#[tokio::main]
async fn main() -> miette::Result<()> {
    let app_opts = AppOptions::parse();
    config_logging(&app_opts);

    match &app_opts.command {
        Commands::Serve { bind } => serve_web(bind.as_deref()).await,
        Commands::RegenerateHtml => regenerate_html_all_posts().await,
        Commands::Worker => run_worker().await,
    }
}

async fn serve_web(bind: Option<&str>) -> miette::Result<()> {
    let config = conf::get_config().map_err(|e| miette!("Error loading config: {e}"))?;
    // The bind option accepts:
    // - TCP addresses like "127.0.0.1:3000" or ":3000"
    // - Unix socket paths like "unix:/tmp/thingsup.sock"
    let addr = get_binding_addr(bind);
    let redis_store = db::get_redis_store()
        .await
        .map_err(|_e| miette!("Error connecting to Redis"))?;

    let client = db::get_gel_client(&config).await.map_err(|e| {
        info!("{e:?}");
        miette!("Failed to create Gel client")
    })?;
    let jinja = config_jinja().into_diagnostic()?;

    // Get Bunny API key and CDN host from config
    let bunny_api_key = conf::get_bunny_api_key(&config)
        .map_err(|e| miette!("Error getting Bunny API key: {e}"))?
        .clone();
    let bunny_cdn_host = conf::get_bunny_cdn_host(&config)
        .map_err(|e| miette!("Error getting Bunny CDN host: {e}"))?
        .clone();

    // Create channel for AI chatbot tracking events
    let (tx, rx) = mpsc::channel::<AIChatbotEvent>(4096);

    // Spawn the consumer task
    tokio::spawn(report_ai_chatbot_visit(
        rx,
        MATOMO_URL.to_string(),
        MATOMO_SITE_ID,
    ));

    let app_state = AppState {
        db: client.clone(),
        jinja,
        bunny_api_key,
        bunny_cdn_host,
    };
    let session_layer = SessionManagerLayer::new(redis_store).with_expiry(Expiry::OnSessionEnd);

    // Auth service
    let backend = Backend { db: client };
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let home_router: Router<AppState> = front::routes::get_router();
    let api_router: Router<AppState> = api::get_router().with_state(app_state.clone());

    // Build middleware layer for AI chatbot tracking (wraps the whole app so it runs first)
    let tracking_tx = Arc::new(tx);
    let tracking_layer =
        middleware::from_fn_with_state(tracking_tx, matomo::ai_chatbot_tracking_middleware);

    let router = Router::new()
        .merge(home_router)
        .nest("/_api", api_router)
        .fallback(front::views::fallback_view)
        .with_state(app_state)
        .layer(auth_layer)
        .layer(TraceLayer::new_for_http());

    let app = router.layer(tracking_layer);

    let main_service = app.into_make_service();
    match addr {
        BindingAddr::Unix(p) => {
            let lt = UnixListener::bind(p).into_diagnostic()?;
            tracing::info!("Listening on {}", addr);
            let perm = Permissions::from_mode(0o664);
            tracing::info!("To set permission {:?}", &perm);
            fs::set_permissions(p, perm).into_diagnostic()?;
            axum::serve(lt, main_service)
                .with_graceful_shutdown(on_shutdown_signal(Some(p.to_path_buf())))
                .await
        }
        BindingAddr::Tcp(s) => {
            let lt = TcpListener::bind(s).await.into_diagnostic()?;
            tracing::info!("Listening on http://{}", addr);
            axum::serve(lt, main_service)
                .with_graceful_shutdown(on_shutdown_signal(None))
                .await
        }
    }
    .into_diagnostic()?;
    Ok(())
}

#[derive(Debug, Clone)]
struct PendingVisit {
    visit_id: matomo::AIChatbotVisitId,
    target: matomo::ChatbotTarget,
    user_agent: String,
    cdt: DateTime<Utc>,
    inserted_at: DateTime<Utc>,
}

/// Background Tokio task that receives AI chatbot events from a channel,
/// merges request + response events, and reports to the Matomo tracking API.
async fn report_ai_chatbot_visit(
    mut rx: mpsc::Receiver<AIChatbotEvent>,
    _matomo_url: String,
    _site_id: u8,
) {
    let mut pending: HashMap<matomo::AIChatbotVisitId, PendingVisit> = HashMap::new();
    let timeout = Duration::from_secs(5);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap_or_default();

    const FLUSH_INTERVAL: Duration = Duration::from_secs(1);
    let mut flush_timer = time::interval(FLUSH_INTERVAL);
    flush_timer.set_missed_tick_behavior(time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            biased;

            event = rx.recv() => match event {
                Some(event) => handle_ai_chatbot_event(&client, &mut pending, event).await,
                None => {
                    flush_stale_visits(&client, &mut pending, None).await;
                    tracing::info!("Chatbot tracking consumer shutting down gracefully");
                    break;
                }
            },

            _ = flush_timer.tick() => {
                flush_stale_visits(&client, &mut pending, Some(timeout)).await;
            }
        }
    }
}

async fn handle_ai_chatbot_event(
    client: &reqwest::Client,
    pending: &mut HashMap<matomo::AIChatbotVisitId, PendingVisit>,
    event: AIChatbotEvent,
) {
    match event {
        AIChatbotEvent::Request(partial) => {
            const PENDING_LIMIT: usize = 4096;
            if pending.len() >= PENDING_LIMIT {
                evict_oldest_pending_visit(client, pending).await;
            }
            pending.insert(
                partial.visit_id.clone(),
                PendingVisit {
                    visit_id: partial.visit_id,
                    target: partial.target,
                    user_agent: partial.user_agent,
                    cdt: partial.cdt,
                    inserted_at: Utc::now(),
                },
            );
        }
        AIChatbotEvent::Response(response) => match pending.remove(&response.visit_id) {
            Some(entry) => {
                send_tracking_request(
                    client,
                    &entry,
                    response.http_status,
                    response.bw_bytes,
                    response.pf_srv,
                )
                .await;
            }
            None => {
                tracing::debug!("Received response for unknown visit {}", response.visit_id);
            }
        },
    }
}

/// When the pending map is full, drop the oldest visit so the map stays bounded.
async fn evict_oldest_pending_visit(
    client: &reqwest::Client,
    pending: &mut HashMap<matomo::AIChatbotVisitId, PendingVisit>,
) {
    let oldest = pending
        .iter()
        .min_by_key(|(_, entry)| entry.inserted_at)
        .map(|(k, _)| k.clone());
    match oldest.and_then(|key| pending.remove(&key)) {
        Some(entry) => send_tracking_request(client, &entry, 0, 0, 0).await,
        None => {}
    }
}

/// Send tracking requests for visits that exceeded `max_age` (or all of them on shutdown).
async fn flush_stale_visits(
    client: &reqwest::Client,
    pending: &mut HashMap<matomo::AIChatbotVisitId, PendingVisit>,
    max_age: Option<Duration>,
) {
    let now = Utc::now();
    let stale: Vec<matomo::AIChatbotVisitId> = pending
        .iter()
        .filter(|(_, entry)| match max_age {
            None => true,
            Some(max_age) => {
                (now - entry.inserted_at).num_milliseconds() > max_age.as_millis() as i64
            }
        })
        .map(|(k, _)| k.clone())
        .collect();

    for visit_id in &stale {
        match pending.remove(visit_id) {
            Some(entry) => send_tracking_request(client, &entry, 0, 0, 0).await,
            None => {}
        }
    }
}

async fn send_tracking_request(
    client: &reqwest::Client,
    entry: &PendingVisit,
    http_status: u16,
    bw_bytes: u32,
    pf_srv: u64,
) {
    let _ = (http_status, bw_bytes, pf_srv);
    let tracking_url = matomo::build_matomo_url(&entry.target, &entry.user_agent, &entry.cdt);

    if let Err(e) = client.get(&tracking_url).send().await {
        tracing::warn!(
            "Failed to send bot tracking event for {}: {}",
            entry.visit_id,
            e
        );
    }
}

async fn regenerate_html_all_posts() -> miette::Result<()> {
    use crate::utils::markdown::markdown_to_html;

    tracing::info!("Regenerating HTML for blog posts...");

    let config = conf::get_config().map_err(|e| miette!("Error loading config: {e}"))?;
    let client = db::get_gel_client(&config).await.map_err(|e| {
        info!("{e:?}");
        miette!("Failed to create Gel client")
    })?;

    // Get all posts with their title and body
    let posts = stores::blog::get_all_posts_for_regeneration(&client)
        .await
        .map_err(|e| miette!("Failed to fetch posts: {e}"))?;

    tracing::info!("Found {} posts to regenerate", posts.len());

    for post in posts {
        let body = post.body.unwrap_or_default();
        let html = markdown_to_html(&body);
        stores::blog::update_post_html(&client, post.id, &html)
            .await
            .map_err(|e| miette!("Failed to update post {}: {}", post.id, e))?;
        println!(
            "Regenerated HTML for post '{}' ({})",
            post.title.blue(),
            post.id
        );
    }

    println!("{}", "HTML regeneration complete!".green());
    Ok(())
}

async fn run_worker() -> miette::Result<()> {
    tracing::info!("Starting background worker...");

    // TODO: Implement worker using apalis_redis
    // This requires proper Redis client configuration compatible with apalis
    tracing::info!("Worker not yet implemented");

    Ok(())
}

async fn on_shutdown_signal(sk: Option<PathBuf>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler!");
    };
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install TERM signal handler")
            .recv()
            .await;
    };
    tracing::debug!("Wait for signals...");
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {}
    };
    tracing::info!("Got signal to terminate. Exiting...");
    if let Some(sk) = sk {
        fs::remove_file(sk).unwrap_or_default();
    }
    tracing::info!("👾 Bye!");
}
