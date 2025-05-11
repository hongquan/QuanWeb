use std::net::{SocketAddr, SocketAddrV4};
use std::str::FromStr;
use std::{env, io, net::Ipv4Addr, path::Path};

use clap::Parser;
use fluent_templates::static_loader;
use minijinja::Environment;
use tracing_subscriber::{
    filter::{EnvFilter, LevelFilter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

use crate::conf::DEFAULT_PORT;
use crate::utils::jinja_extra;
use crate::{consts::UNCATEGORIZED_URL, types::BindingAddr};

#[derive(Debug, Clone, Parser)]
#[command(author, version, about)]
pub struct AppOptions {
    #[arg(
        short,
        long,
        help = "Network address to bind, can be <port>, <ip:port> or Unix socket in form of <unix:/path/to/file>"
    )]
    pub bind: Option<String>,
    #[arg(short, action = clap::ArgAction::Count, help = "Verbosity")]
    pub verbose: u8,
}

/// Test if current process is connected with journald
pub fn is_journald_connected() -> bool {
    // In desktop (development): JOURNAL_STREAM and TERM are set. We want to log to console.
    // In service: JOURNAL_STREAM is set. TERM is set if stderr is connected to tty instead of journald.
    if env::var_os("JOURNAL_STREAM").is_none() {
        return false;
    }
    let term_available = env::var("TERM").is_ok_and(|s| !s.is_empty());
    !term_available
}

pub fn config_logging(app_opt: &AppOptions) {
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
    let directives = format!("quanweb={level},tower_http={level}");
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .parse_lossy(directives);

    let registry = tracing_subscriber::registry().with(filter);

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
    jinja.set_loader(jinja_extra::get_embedded_template);
    Ok(jinja)
}

pub fn get_listening_addr() -> Ipv4Addr {
    match env::var("CARGO") {
        Ok(_) => Ipv4Addr::UNSPECIFIED,
        Err(_) => Ipv4Addr::LOCALHOST,
    }
}

pub fn get_binding_addr(bind_opt: Option<&str>) -> BindingAddr {
    let addr = if let Some(s) = bind_opt {
        if let Some(sk_path) = s.strip_prefix("unix:") {
            Some(BindingAddr::Unix(Path::new(sk_path)))
        } else if s.contains(':') {
            SocketAddr::from_str(s).ok().map(BindingAddr::Tcp)
        } else {
            None
        }
    } else {
        None
    };
    addr.unwrap_or_else(|| {
        BindingAddr::Tcp(SocketAddr::V4(SocketAddrV4::new(
            get_listening_addr(),
            DEFAULT_PORT,
        )))
    })
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
