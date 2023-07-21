pub mod structs;
pub mod posts;
pub mod views;
pub mod auth;
pub mod paging;
pub mod routes;
pub mod errors;
pub mod macros;
pub mod minors;

#[cfg(test)]
pub mod tests;

pub use routes::get_router;
