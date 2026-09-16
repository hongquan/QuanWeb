# Justfile for QuanWeb project

# Default recipe to show available commands
default:
    @just --list

# Generate CSS using the built-in binary
generate-css:
    cargo run --bin css-gen

# Watch and regenerate CSS on changes
watch-css:
    cargo watch -w minijinja -w src -w static/js -w encre.toml -x "run --bin css-gen"

# Build the Rust backend
build-backend:
    cargo build --release --all-features

# Run the development server
dev:
    cargo run --all-features
