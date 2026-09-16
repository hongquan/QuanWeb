use std::fs;

use miette::IntoDiagnostic;
use owo_colors::OwoColorize;

use encre_css::{Config, generate};

const INPUT_GLOBS: &[&str] = &[
    "minijinja/**/*.jinja",
    "src/**/*.rs",
    "static/js/*.js",
];

#[tokio::main]
async fn main() -> miette::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Generating CSS with EncreCSS...");

    let mut config = Config::from_file("encre.toml").into_diagnostic()?;
    encre_css_typography::register(&mut config);

    let mut buffer = String::new();
    for glob in INPUT_GLOBS {
        for entry in glob::glob(glob).map_err(|e| miette::miette!("Invalid glob {glob}: {e}"))? {
            let path = entry.into_diagnostic()?;
            if path.is_file() {
                buffer.push_str(&fs::read_to_string(path).into_diagnostic()?);
            }
        }
    }

    let css = generate([buffer.as_str()], &config);
    fs::write("static/css/built-tailwind.css", css).into_diagnostic()?;

    println!(
        "{}",
        "CSS generated at static/css/built-tailwind.css".green()
    );
    Ok(())
}
