use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(author, version, about)]
pub struct AppOptions {
    #[arg(short, action = clap::ArgAction::Count)]
    pub verbose: u8,
}
