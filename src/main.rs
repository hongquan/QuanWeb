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

use std::net::SocketAddr;

use auth::backend::Backend;
use axum::routing::Router;
use axum_login::AuthManagerLayerBuilder;
use clap::Parser;
use miette::{miette, IntoDiagnostic};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tower_sessions::SessionManagerLayer;
use tracing::info;

use thingsup::{config_jinja, config_logging, get_binding_addr, get_listening_addr, AppOptions};
use types::AppState;

#[tokio::main]
async fn main() -> miette::Result<()> {
    let app_opts = AppOptions::parse();
    config_logging(&app_opts);
    let redis_store = db::get_redis_store()
        .await
        .map_err(|_e| miette!("Error connecting to Redis"))?;

    let config = conf::get_config().map_err(|e| miette!("Error loading config: {e}"))?;
    let client = db::get_edgedb_client(&config).await.map_err(|e| {
        info!("{e:?}");
        miette!("Failed to create EdgeDB client")
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

    let addr_result = match &app_opts.bind {
        Some(saddr) => get_binding_addr(saddr),
        None => {
            let port = conf::get_listening_port(&config);
            Ok(SocketAddr::from((get_listening_addr(), port)))
        }
    };
    let main_service = app.into_make_service();
    match addr_result {
        Ok(addr) => {
            tracing::info!("Listening on http://{}", addr);
            let listerner = TcpListener::bind(addr).await.into_diagnostic()?;
            axum::serve(listerner, main_service)
                .await
                .into_diagnostic()?;
        }
        _ => {
            unimplemented!()
        }
    }
    Ok(())
}
