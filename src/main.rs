mod api;
mod auth;
mod conf;
mod consts;
mod db;
mod errors;
mod front;
mod models;
mod stores;
mod thingsup;
mod types;
mod utils;

use std::{fs, path::PathBuf};

use auth::backend::Backend;
use axum::routing::Router;
use axum_login::AuthManagerLayerBuilder;
use clap::Parser;
use miette::{IntoDiagnostic, miette};
use tokio::net::{TcpListener, UnixListener};
use tokio::signal;
use tower_http::trace::TraceLayer;
use tower_sessions::SessionManagerLayer;
use tracing::info;

use thingsup::{AppOptions, config_jinja, config_logging, get_binding_addr};
use types::{AppState, BindingAddr};

#[tokio::main]
async fn main() -> miette::Result<()> {
    let app_opts = AppOptions::parse();
    config_logging(&app_opts);
    let config = conf::get_config().map_err(|e| miette!("Error loading config: {e}"))?;
    let addr = get_binding_addr(app_opts.bind.as_deref());
    let redis_store = db::get_redis_store()
        .await
        .map_err(|_e| miette!("Error connecting to Redis"))?;

    let client = db::get_gel_client(&config).await.map_err(|e| {
        info!("{e:?}");
        miette!("Failed to create Gel client")
    })?;
    let jinja = config_jinja().into_diagnostic()?;
    let app_state = AppState {
        db: client.clone(),
        jinja,
    };
    let session_layer = SessionManagerLayer::new(redis_store);

    // Auth service
    let backend = Backend { db: client };
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let home_router: Router<AppState> = front::routes::get_router();
    let api_router: Router<AppState> = api::get_router().with_state(app_state.clone());

    let app = Router::new()
        .merge(home_router)
        .nest("/_api", api_router)
        .fallback(front::views::fallback_view)
        .with_state(app_state)
        .layer(auth_layer)
        .layer(TraceLayer::new_for_http());

    let main_service = app.into_make_service();
    match addr {
        BindingAddr::Unix(p) => {
            let lt = UnixListener::bind(p).into_diagnostic()?;
            tracing::info!("Listening on {}", addr);
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

async fn on_shutdown_signal<'a>(sk: Option<PathBuf>) {
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
