use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use edgedb_protocol::common::Cardinality as Cd;
use edgedb_protocol::value::Value as EValue;
use indexmap::indexmap;
use miette::{miette, IntoDiagnostic, Result};
use syntect::highlighting::ThemeSet;
use syntect::html::css_for_theme_with_class_style;
use tokio;
use tracing::debug;
use tracing_subscriber::{
    filter::{EnvFilter, LevelFilter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use uuid::Uuid;

use quanweb::conf;
use quanweb::consts::SYNTECT_CLASS_STYLE;
use quanweb::db;
use quanweb::types::conversions::edge_object_from_pairs;

const OUTPUT_PATH: &str = "static/css/syntect.css";
const SYNTECT_THEME: &str = "base16-ocean.dark";

/// Some tools for QuanWeb
#[derive(Debug, Clone, Parser)]
struct ToolOptions {
    #[arg(short, action = clap::ArgAction::Count)]
    pub verbose: u8,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone, Subcommand)]
enum Commands {
    /// Generate CSS for code highlighting from Syntect
    GenSyntectCSS,
    TryUpdateCategory {
        id: Uuid,
    },
}

fn config_logging() {
    let directives = format!("{level}", level = LevelFilter::DEBUG);
    let filter = EnvFilter::new(directives);
    let registry = tracing_subscriber::registry().with(filter);
    registry.with(tracing_subscriber::fmt::layer()).init();
}

fn gen_syntect_css() -> Result<()> {
    let theme_set = ThemeSet::load_defaults();
    let theme = theme_set
        .themes
        .get(SYNTECT_THEME)
        .ok_or(miette!("Theme not found"))?;
    let content =
        css_for_theme_with_class_style(theme, SYNTECT_CLASS_STYLE).map_err(|e| miette!("{e}"))?;
    let path = PathBuf::from(OUTPUT_PATH);
    if path.exists() {
        fs::remove_file(&path).into_diagnostic()?;
    }
    eprintln!("To write to {OUTPUT_PATH}");
    fs::write(path, content).into_diagnostic()?;
    eprintln!("🎉 Done!");
    Ok(())
}

async fn update_with_tuple(id: Uuid, client: &edgedb_tokio::Client) -> Result<()> {
    let q_simple = "UPDATE BlogCategory FILTER .id = <uuid>$0 SET { title := <str>$1 }";
    let title = "Test with tuple".to_string();
    let t_args = (id, title);
    tracing::debug!("To query: {}", q_simple);
    tracing::debug!("With args: {:#?}", t_args);
    client.execute(&q_simple, &t_args).await.map_err(|e| {
        tracing::error!("{:#?}", e);
        miette!("Error querying.")
    })?;
    eprintln!("🎉 Done!");
    Ok(())
}

async fn update_with_params(id: Uuid, client: &edgedb_tokio::Client) -> Result<()> {
    let title = "Test with params".to_string();
    let pairs = indexmap!(
        "id" => (Some(EValue::Uuid(id)), Cd::One),
        "title" => (Some(EValue::Str(title)), Cd::One),
    );
    let args = edge_object_from_pairs(pairs);
    let q_simple = "UPDATE BlogCategory FILTER .id = <uuid>$id SET { title := <str>$title }";
    tracing::debug!("To query: {}", q_simple);
    tracing::debug!("With args: {:#?}", args);
    client.execute(&q_simple, &args).await.map_err(|e| {
        tracing::error!("{:#?}", e);
        miette!("Error querying.")
    })?;
    eprintln!("🎉 Done!");
    Ok(())
}

async fn try_update_category(id: Uuid) -> Result<()> {
    eprintln!("id: {}", id);
    let config = conf::get_config().map_err(|e| miette!("Error loading config: {e}"))?;
    let client = db::get_edgedb_client(&config).await.map_err(|e| {
        debug!("{e:?}");
        miette!("Failed to create EdgeD client")
    })?;
    update_with_tuple(id, &client).await?;
    update_with_params(id, &client).await?;
    eprintln!("🎉 Done!");
    Ok(())
}

fn main() -> Result<()> {
    let opts = ToolOptions::parse();
    config_logging();
    match opts.command {
        Commands::GenSyntectCSS => {
            gen_syntect_css()?;
        }
        Commands::TryUpdateCategory { id } => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async { try_update_category(id).await })?;
        }
    }
    Ok(())
}
