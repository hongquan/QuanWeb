use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use syntect::html::{ClassStyle, css_for_theme_with_class_style};
use syntect::highlighting::ThemeSet;
use miette::{miette, Result, IntoDiagnostic};

use quanweb::consts::SYNTECT_THEME;

const OUTPUT_PATH: &str = "static/css/syntect.css";

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
    GenSyntectCSS
}

fn gen_syntect_css() -> Result<()> {
    let theme_set = ThemeSet::load_defaults();
    let theme = theme_set.themes.get(SYNTECT_THEME).ok_or(miette!("Theme not found"))?;
    let content = css_for_theme_with_class_style(theme, ClassStyle::SpacedPrefixed { prefix: "st-" }).map_err(|e| miette!("{e}"))?;
    let path = PathBuf::from(OUTPUT_PATH);
    if path.exists() {
        fs::remove_file(&path).into_diagnostic()?;
    }
    eprintln!("To write to {OUTPUT_PATH}");
    fs::write(path, content).into_diagnostic()?;
    eprintln!("🎉 Done!");
    Ok(())
}

fn main() -> Result<()> {
    let opts = ToolOptions::parse();
    match opts.command {
        Commands::GenSyntectCSS => {
            gen_syntect_css()?;
        }
    }
    Ok(())
}
