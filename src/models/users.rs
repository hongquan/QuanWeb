use uuid::Uuid;
use field_names::FieldNames;
use axum_login::{AuthUser, secrecy::SecretVec};
use edgedb_derive::Queryable;
use serde::{Serialize, Deserialize};
use atom_syndication::{Person, PersonBuilder};

use super::feeds::JsonAuthor;
use crate::types::EdgeSelectable;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, Queryable)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub is_active: bool,
    pub is_superuser: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, Queryable, FieldNames)]
pub struct MiniUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

impl EdgeSelectable for MiniUser {
    fn fields_as_shape() -> String {
        let fields = Self::FIELDS.join(", ");
        format!("{{ {fields} }}")
    }
}

impl From<MiniUser> for Person {
    fn from(user: MiniUser) -> Self {
        PersonBuilder::default()
            .name(user.username)
            .email(Some(user.email))
            .build()
    }
}

impl From<MiniUser> for JsonAuthor {
    fn from(user: MiniUser) -> Self {
        JsonAuthor {
            name: Some(user.username),
            url: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Role {
    Admin,
}

impl AuthUser<Uuid, Role> for User {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_password_hash(&self) -> SecretVec<u8> {
        SecretVec::new(self.password.clone().into())
    }

    fn get_role(&self) -> Option<Role> {
        Some(Role::Admin)
    }
}
