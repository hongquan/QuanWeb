pub mod backend;
pub mod structs;

use backend::Backend;

pub type AuthSession = axum_login::AuthSession<Backend>;
