use std::{env, io};

use clap::Parser;
use faccess::PathExt;
use tracing_subscriber::{
    filter::{EnvFilter, LevelFilter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use minijinja::{path_loader, Environment};
use fluent_templates::static_loader;

use crate::consts::{TEMPLATE_DIR, UNCATEGORIZED_URL};
use crate::utils::jinja_extra;

#[derive(Debug, Clone, Parser)]
#[command(author, version, about)]
pub struct AppOptions {
    #[arg(short, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

/// Test if current process is connected with journald
pub fn is_journald_connected() -> bool {
    // In desktop (development): JOURNAL_STREAM and TERM are set. We want to log to console.
    // In service: JOURNAL_STREAM is set. TERM is set if stderr is connected to tty instead of journald.
    if env::var_os("JOURNAL_STREAM").is_none() {
        return false
    }
    let term_available = env::var("TERM").map_or(false, |s| !s.is_empty());
    !term_available
}

pub fn config_logging(app_opt: AppOptions) {
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

    let registry = tracing_subscriber::registry()
        .with(filter);

    if is_journald_connected() {
        if let Ok(journald_layer) = tracing_journald::layer() {
            registry.with(journald_layer).init();
        }
    } else {
        registry.with(tracing_subscriber::fmt::layer()).init();
    }

}

pub fn config_jinja() -> Result<Environment<'static>, io::Error> {
    let mut jinja = Environment::new();
    jinja.add_filter("debug_value", jinja_extra::debug_value);
    jinja.add_filter("post_detail_url", jinja_extra::post_detail_url);
    jinja.add_filter("category_url", jinja_extra::category_url);
    jinja.add_function("gen_element_attr", jinja_extra::gen_element_attr);
    jinja.add_function("add_url_param", jinja_extra::add_url_param);
    jinja.add_function("_f", jinja_extra::fluent);
    jinja.add_filter("striptags", jinja_extra::striptags);
    jinja.add_global("UNCATEGORIZED_URL", UNCATEGORIZED_URL);
    jinja.add_global("GIT_REVISION", env!("GIT_REVISION"));
    #[cfg(debug_assertions)]
    jinja.add_global("running_locally", true);
    let template_dir = env::current_dir()?.join(TEMPLATE_DIR);
    if !(template_dir.is_dir() && template_dir.readable()) {
        return Err(io::Error::from(io::ErrorKind::PermissionDenied))
    }
    jinja.set_loader(path_loader(&template_dir));
    Ok(jinja)
}

pub fn get_listening_addr() -> [u8; 4] {
    match env::var("CARGO") {
        Ok(_) => [0, 0, 0, 0],
        Err(_) => [127, 0, 0, 1],
    }
}

static_loader! {
    pub static LOCALES = {
        locales: "./locales",
        fallback_language: "en",
        customise: |bundle| {
            bundle.set_use_isolating(false);
        },
    };
}
