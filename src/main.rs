mod conf;
mod api;
mod auth;
mod consts;
mod db;
mod models;
mod retrievers;
mod types;
mod utils;
mod views;

use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::routing::{get, Router};
use axum_login::{axum_sessions::SessionLayer, AuthLayer};
use miette::miette;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use auth::store::EdgeDbStore;
use types::{AppState, SharedState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "quanweb=debug,axum_login=debug,tower_http=debug,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let redis_store = db::get_redis_store()
        .await
        .map_err(|_e| miette!("Error connecting to Redis"))?;

    let config = conf::get_config().map_err(|e| miette!("Error loading config: {e}"))?;
    let secret_bytes = conf::get_secret_bytes(&config).map_err(|e| miette!("Error getting secret bytes: {e}"))?;
    let client = db::get_edgedb_client().await?;
    let shared_state = Arc::new(AppState { db: client.clone() });
    let session_layer = SessionLayer::new(redis_store, &secret_bytes).with_secure(false);
    let user_store: EdgeDbStore<models::User> = EdgeDbStore::new(client);
    let auth_layer = AuthLayer::new(user_store, &secret_bytes);

    let api_router: Router<SharedState> = api::get_router().with_state(Arc::clone(&shared_state));

    let app = Router::new()
        .route("/", get(views::base::root))
        .nest("/api", api_router)
        .with_state(shared_state)
        .layer(auth_layer)
        .layer(session_layer)
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Listening on http://{}", addr);

    // TODO: Support Unix domain socket with hyperlocal
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
