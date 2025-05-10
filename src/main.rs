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
use tokio_listener::axum07::serve as axum_serve;
use tokio_listener::{Listener, ListenerAddress, SystemOptions, UnixChmodVariant, UserOptions};
use tower_http::trace::TraceLayer;
use tower_sessions::SessionManagerLayer;
use tracing::info;

use thingsup::{config_jinja, config_logging, get_binding_addr, get_listening_addr, AppOptions};
use types::AppState;

#[tokio::main]
async fn main() -> miette::Result<()> {
    let app_opts = AppOptions::parse();
    config_logging(&app_opts);
    let config = conf::get_config().map_err(|e| miette!("Error loading config: {e}"))?;
    let addr = match &app_opts.bind {
        Some(saddr) => get_binding_addr(saddr).map_err(|e| miette!("{e}")),
        None => {
            let port = conf::get_listening_port(&config);
            Ok(ListenerAddress::Tcp(SocketAddr::from((
                get_listening_addr(),
                port,
            ))))
        }
    }?;
    let (redis_store, redis_conn) = db::get_redis_store()
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
    tracing::info!("Listening on http://{}", addr);
    let sys_opts = SystemOptions::default();
    let mut usr_opts = UserOptions::default();
    usr_opts.unix_listen_unlink = true;
    usr_opts.unix_listen_chmod = Some(UnixChmodVariant::Group);
    let listerner = Listener::bind(&addr, &sys_opts, &usr_opts)
        .await
        .into_diagnostic()?;
    axum_serve(listerner, main_service)
        .await
        .into_diagnostic()?;
    redis_conn
        .await
        .map_err(|_e| miette!("Redis error."))?
        .map_err(|_e| miette!("{}", _e))?;
    Ok(())
}
