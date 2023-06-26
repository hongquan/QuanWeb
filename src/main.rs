mod consts;
mod views;
mod models;
mod auth;
mod db;
mod retrievers;
mod types;
mod api;

use std::net::SocketAddr;
use std::sync::Arc;

use rand::Rng;
use axum::routing::get;
use axum_named_routes::NamedRouter;
use axum_login::{
    axum_sessions::{async_session::MemoryStore, SessionLayer},
    AuthLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower_http::trace::TraceLayer;

use auth::store::EdgeDbStore;
use types::AppState;

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
    let shared_state = Arc::new(AppState {
        db: client.clone(),
    });
    let session_store = MemoryStore::new();
    let session_layer = SessionLayer::new(session_store, &secret).with_secure(false);
    let user_store: EdgeDbStore<models::User> = EdgeDbStore::new(client);
    let auth_layer = AuthLayer::new(user_store, &secret);

    let api_router = api::get_router(Arc::clone(&shared_state));

    let mut app = NamedRouter::new()
        .route("index", "/", get(views::base::root))
        .with_state(shared_state)
        .nest("api", "/api", api_router);

    app = app
        .layer(auth_layer)
        .layer(session_layer)
        .layer(TraceLayer::new_for_http());


    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Listening on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
