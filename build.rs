use std::env;
use std::process::Command;

fn main() {
    // Build frontend also if we are in release mode
    if env::var("PROFILE") == Ok("release".into()) {
        eprintln!("To build frontend in admin...");
        Command::new("yarn")
            .args(&["--cwd", "admin", "build"])
            .status()
            .expect("Failed to build frontend");
    }
    crate_git_revision::init();
}
