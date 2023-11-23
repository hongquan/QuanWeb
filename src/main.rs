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
use std::os::unix::fs::PermissionsExt;
use std::{fs, path::Path};

use auth::backend::Backend;
use axum::routing::Router;
use axum::{error_handling::HandleErrorLayer, BoxError};
use axum_login::AuthManagerLayerBuilder;
use clap::Parser;
use http::StatusCode;
use hyperlocal::UnixServerExt;
use miette::{miette, IntoDiagnostic};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tower_sessions::SessionManagerLayer;

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
    let client = db::get_edgedb_client(&config).await?;
    let jinja = config_jinja().into_diagnostic()?;
    let app_state = AppState {
        db: client.clone(),
        jinja,
    };
    let session_layer = SessionManagerLayer::new(redis_store);

    // Auth service
    let backend = Backend { db: client };
    let auth_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_e: BoxError| async {
            StatusCode::INTERNAL_SERVER_ERROR
        }))
        .layer(AuthManagerLayerBuilder::new(backend, session_layer).build());

    let home_router: Router<AppState> = front::routes::get_router();
    let api_router: Router<AppState> = api::get_router().with_state(app_state.clone());

    let app = Router::new()
        .merge(home_router)
        .nest("/_api", api_router)
        .fallback(front::views::fallback_view)
        .with_state(app_state)
        .layer(auth_service)
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
            axum::Server::bind(&addr)
                .serve(main_service)
                .await
                .into_diagnostic()?;
        }
        _ => {
            let original_bind = app_opts.bind.unwrap_or("web.sock".into());
            let path = Path::new(&original_bind);
            if path.exists() {
                std::fs::remove_file(path).into_diagnostic()?;
            }
            tracing::info!("Listening on unix:{}", path.display());
            let server = axum::Server::bind_unix(path).into_diagnostic()?;
            let perm = fs::Permissions::from_mode(0o664);
            fs::set_permissions(path, perm).into_diagnostic()?;
            server.serve(main_service).await.into_diagnostic()?;
        }
    }
    Ok(())
}
