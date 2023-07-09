use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(author, version, about)]
pub struct AppOptions {
    #[arg(short, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[allow(dead_code)]
impl AppOptions {
    pub fn get_log_level(&self) -> &str {
        match self.verbose {
            0 => "warning",
            1 => "info",
            2 => "debug",
            _ => "trace",
        }
    }
}
