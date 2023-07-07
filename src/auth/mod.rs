pub mod store;
pub mod structs;

use uuid::Uuid;
use axum_login::extractors::AuthContext;

use crate::models::{User, Role};
use store::EdgeDbStore;

pub type Auth = AuthContext<Uuid, User, EdgeDbStore<User>, Role>;
