use uuid::Uuid;

use axum_login::{AuthUser, secrecy::SecretVec};
use edgedb_derive::Queryable;

#[derive(Debug, Default, Clone, PartialEq, Queryable)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password: String,
    pub is_active: bool,
    pub is_superuser: bool,
}

impl AuthUser<Uuid> for User {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_password_hash(&self) -> SecretVec<u8> {
        SecretVec::new(self.password.clone().into())
    }
}
