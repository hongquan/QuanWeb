mod api;
mod auth;
mod conf;
mod consts;
mod db;
mod errors;
mod models;
mod stores;
mod types;
mod utils;
mod cli;
mod front;

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::Parser;
use axum::routing::Router;
use axum_login::{axum_sessions::SessionLayer, AuthLayer};
use miette::miette;
use minijinja::{path_loader, Environment};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use auth::store::EdgeDbStore;
use types::AppState;
use utils::jinja_extra;

const TEMPLATE_DIR: &str = "minijinja";

fn config_jinja() -> Environment<'static> {
    let mut jinja = Environment::new();
    jinja.add_function("post_detail_url", jinja_extra::post_detail_url);
    jinja.add_function("gen_element_attr", jinja_extra::gen_element_attr);
    jinja.add_function("add_url_param", jinja_extra::add_url_param);
    #[cfg(debug_assertions)]
    jinja.add_global("running_locally", true);
    let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(TEMPLATE_DIR);
    jinja.set_loader(path_loader(&template_path));
    jinja
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    let _app_opts = cli::AppOptions::parse();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "quanweb=debug,axum_login=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let redis_store = db::get_redis_store()
        .await
        .map_err(|_e| miette!("Error connecting to Redis"))?;

    let config = conf::get_config().map_err(|e| miette!("Error loading config: {e}"))?;
    let secret_bytes =
        conf::get_secret_bytes(&config).map_err(|e| miette!("Error getting secret bytes: {e}"))?;
    let client = db::get_edgedb_client().await?;
    let jinja = config_jinja();
    let app_state = AppState {
        db: client.clone(),
        jinja,
    };
    let session_layer = SessionLayer::new(redis_store, &secret_bytes).with_secure(false);
    let user_store: EdgeDbStore<models::User> = EdgeDbStore::new(client);
    let auth_layer = AuthLayer::new(user_store, &secret_bytes);

    let home_router: Router<AppState> = front::routes::get_router();
    let api_router: Router<AppState> = api::get_router().with_state(app_state.clone());

    let app = Router::new()
        .merge(home_router)
        .nest("/_api", api_router)
        .fallback(front::views::fallback_view)
        .with_state(app_state)
        .layer(auth_layer)
        .layer(session_layer)
        .layer(TraceLayer::new_for_http());

    let port = conf::get_listening_port(&config);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Listening on http://{}", addr);

    // TODO: Support Unix domain socket with hyperlocal
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
