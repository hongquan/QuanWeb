pub mod auth;
pub mod errors;
pub mod files;
pub mod macros;
pub mod minors;
pub mod paging;
pub mod posts;
pub mod routes;
pub mod structs;
pub mod users;
pub mod views;

#[cfg(test)]
pub mod tests;

pub use routes::get_router;
