mod api;
mod auth;
mod conf;
mod consts;
mod db;
mod errors;
mod front;
mod models;
mod stores;
mod types;
mod utils;
mod thingsup;

use std::net::SocketAddr;

use axum::routing::Router;
use axum_login::{axum_sessions::SessionLayer, AuthLayer};
use clap::Parser;
use miette::{miette, IntoDiagnostic};
use tower_http::trace::TraceLayer;

use auth::store::EdgeDbStore;
use types::AppState;
use thingsup::{AppOptions, config_jinja, config_logging, get_listening_addr};

#[tokio::main]
async fn main() -> miette::Result<()> {
    let app_opts = AppOptions::parse();
    config_logging(app_opts);
    let redis_store = db::get_redis_store()
        .await
        .map_err(|_e| miette!("Error connecting to Redis"))?;

    let config = conf::get_config().map_err(|e| miette!("Error loading config: {e}"))?;
    let secret_bytes =
        conf::get_secret_bytes(&config).map_err(|e| miette!("Error getting secret bytes: {e}"))?;
    let client = db::get_edgedb_client(&config).await?;
    let jinja = config_jinja().into_diagnostic()?;
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
    let addr = SocketAddr::from((get_listening_addr(), port));
    tracing::info!("Listening on http://{}", addr);

    // TODO: Support Unix domain socket with hyperlocal
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
