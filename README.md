# Personal website #

This is my personal website. At first, it only has a blog, with Markdown being input format.

The website is programmed in Rust (used to be Python, but was rewritten as an exercise to learn Rust).

## Tech stack

- Backend:

  + Programming language: Rust
  + Framework: [Axum](https://crates.io/crates/axum)
  + Database: [Gel](https://www.geldata.com), Redis
  + Localization: [Fluent](https://projectfluent.org/)

- Frontend:

  + Server-side rendering: [MiniJinja](https://crates.io/crates/minijinja)
  + SPA (for Admin): [VueJS](https://vuejs.org/), TypeScript, [TailwindCSS](https://tailwindcss.com/)

## Backend Folder Structure

The backend application follows a modular structure with the following main directories at the root level:

- `src/` - Main source code directory
  - `api/` - API-related code including routes, handlers, and data structures
  - `auth/` - Authentication system including backend implementation and data structures
  - `conf/` - Configuration handling and settings
  - `consts/` - Application constants
  - `db/` - Database connection and client setup
  - `errors/` - Error handling and custom error types
  - `front/` - Frontend-related code including routes and views
  - `models/` - Data models and structures used throughout the application
  - `stores/` - Data access layer for interacting with the database
  - `thingsup/` - Application startup and configuration utilities
  - `types/` - Custom type definitions and extensions
  - `utils/` - Utility functions and helpers used across the application

- `static/` - Static assets like CSS, JavaScript, and images
- `minijinja/` - MiniJinja templates for server-side rendering
- `locales/` - Localization files for internationalization

## Frontend Admin App (Lustre) Structure

The admin frontend is a single-page application built with the Lustre framework and is located in the `ladmin/` directory:

- `src/` - Main source code directory
  - `consts.gleam` - Application constants
  - `core.gleam` - Core data types and message definitions
  - `decoders.gleam` - JSON decoders for API responses
  - `ffi.gleam` - Foreign function interface to JavaScript
  - `models.gleam` - Application state model and helper functions
  - `routes.gleam` - Routing logic and URL parsing
  - `store.gleam` - Local storage persistence for user session
  - `updates.gleam` - Message handling and state update logic
  - `views/` - View components organized by feature
    - `blog_categories.gleam` - Category management views
    - `blog_posts.gleam` - Post management views
    - `books.gleam` - Book management views
    - `presentations.gleam` - Presentation management views
    - `simple.gleam` - Simple components like login page
  - `icons/` - Icon components
  - `element.ffi.mjs` - JavaScript interop functions

