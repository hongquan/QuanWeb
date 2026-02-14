# Justfile for QuanWeb project

# Default recipe to show available commands
default:
    @just --list

# Build CSS with EncreCSS
build-css:
    encrecss build -c encre.toml -o static/css/built-tailwind.css

# Watch and rebuild CSS on changes
watch-css:
    encrecss build -c encre.toml -o static/css/built-tailwind.css -w

# Build the Rust backend
build-backend:
    cargo build --release --all-features

# Run the development server
dev:
    cargo run --all-features
