use std::env;
use std::process::Command;

fn main() {
    // Build the CSS file also if we are in release mode
    if env::var("PROFILE") == Ok("release".into()) {
        eprintln!("To build static files...");
        Command::new("encrecss")
            .args(["build", "-o", "static/css/built-tailwind.css"])
            .status()
            .expect("Failed to build static files!");
    }
    crate_git_revision::init();
}
