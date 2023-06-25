mod consts;
mod views;
mod models;
mod auth;
mod db;
mod retrievers;

use std::net::SocketAddr;

use rand::Rng;
use axum::routing::{get, post};
use axum_named_routes::NamedRouter;
use axum_login::{
    axum_sessions::{async_session::MemoryStore, SessionLayer},
    AuthLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use auth::store::EdgeDbStore;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "quanweb=debug,tower_http=debug,axum::rejection=trace".into()
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let secret = rand::thread_rng().gen::<[u8; 64]>();
    let client = match db::get_edgedb_client().await {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Error connecting to EdgeDB: {}", e);
            return;
        }
    };
    let session_store = MemoryStore::new();
    let session_layer = SessionLayer::new(session_store, &secret).with_secure(false);
    let user_store: EdgeDbStore<models::User> = EdgeDbStore::new(client);
    let auth_layer = AuthLayer::new(user_store, &secret);

    let api_router = views::get_api_router();

    let app = NamedRouter::new()
        .route("index", "/", get(views::root))
        .route("api-login", "/api/login", post(auth::views::login))
        .route("api-login-short", "/api/login-short", post(auth::views::login_short))
        .nest("api", "/api", api_router)
        .layer(auth_layer)
        .layer(session_layer);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Listening on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
