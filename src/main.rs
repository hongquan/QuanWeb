mod api;
mod auth;
mod cli;
mod conf;
mod consts;
mod db;
mod errors;
mod front;
mod models;
mod stores;
mod types;
mod utils;

use std::env;
use std::io;
use std::net::SocketAddr;

use faccess::PathExt;
use axum::routing::Router;
use axum_login::{axum_sessions::SessionLayer, AuthLayer};
use clap::Parser;
use miette::{miette, IntoDiagnostic};
use minijinja::{path_loader, Environment};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{
    filter::{EnvFilter, LevelFilter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

use auth::store::EdgeDbStore;
use cli::AppOptions;
use types::AppState;
use utils::jinja_extra;

const TEMPLATE_DIR: &str = "minijinja";

fn config_jinja() -> Result<Environment<'static>, io::Error> {
    let mut jinja = Environment::new();
    jinja.add_function("post_detail_url", jinja_extra::post_detail_url);
    jinja.add_function("gen_element_attr", jinja_extra::gen_element_attr);
    jinja.add_function("add_url_param", jinja_extra::add_url_param);
    jinja.add_filter("striptags", jinja_extra::striptags);
    #[cfg(debug_assertions)]
    jinja.add_global("running_locally", true);
    let template_dir = env::current_dir()?.join(TEMPLATE_DIR);
    if !(template_dir.is_dir() && template_dir.readable()) {
        return Err(io::Error::from(io::ErrorKind::PermissionDenied))
    }
    jinja.set_loader(path_loader(&template_dir));
    Ok(jinja)
}

fn config_logging(app_opt: AppOptions) {
    // If run by "cargo run", we want to see debug logs.
    let run_by_cargo = env::var("CARGO").is_ok();
    let level = if run_by_cargo {
        LevelFilter::DEBUG
    } else {
        match app_opt.verbose {
            0 => LevelFilter::WARN,
            1 => LevelFilter::INFO,
            2 => LevelFilter::DEBUG,
            _ => LevelFilter::TRACE,
        }
    };
    let command_directives = format!("quanweb={level},axum_login={level},tower_http={level}");
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .parse(command_directives)
        .unwrap();
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    let app_opts = cli::AppOptions::parse();
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
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Listening on http://{}", addr);

    // TODO: Support Unix domain socket with hyperlocal
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
