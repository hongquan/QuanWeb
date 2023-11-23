use async_trait::async_trait;
use axum_login::{AuthUser, AuthnBackend, UserId};
use djangohashers::check_password;
use tracing::info;

use crate::models::User;
use crate::stores::user::get_user_by_email;

#[derive(Clone, Debug)]
pub struct Backend {
    pub db: edgedb_tokio::Client,
}

#[derive(Clone, Debug)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

impl AuthUser for User {
    type Id = uuid::Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password.as_bytes()
    }
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = edgedb_errors::Error;

    async fn authenticate(
        &self,
        cred: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user = if let Some(user) = get_user_by_email(&cred.email, &self.db).await? {
            let right_passwd = check_password(&cred.password, &user.password).unwrap_or_default();
            info!("Right password? {}", right_passwd);
            right_passwd.then_some(user)
        } else {
            info!("User with {} is not found.", cred.email);
            None
        };
        Ok(user)
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        info!("To load user with ID {:?}", user_id);
        let q = "SELECT User {id, username, email, password, is_active, is_superuser} FILTER .id = <uuid>$0";
        let user: Option<User> = self.db.query_single(q, &(user_id,)).await?;
        Ok(user)
    }
}
