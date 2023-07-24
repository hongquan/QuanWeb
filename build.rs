use std::env;
use std::process::Command;

fn main() {
    // Build the CSS file also if we are in release mode
    if env::var("PROFILE") == Ok("release".into()) {
        eprintln!("To build static files...");
        Command::new("yarn")
            .args(&["build-tailwind"])
            .status()
            .expect("Failed to build static files!");
    }
    crate_git_revision::init();
}
