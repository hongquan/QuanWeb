use std::hash::Hash;
use std::marker::PhantomData;

use async_trait::async_trait;
use axum_login::{UserStore, AuthUser};
use edgedb_protocol::{query_arg::ScalarArg, queryable::Queryable};
use eyre::Error;

#[derive(Clone, Debug)]
pub struct EdgeDbStore<User> {
    client: edgedb_tokio::Client,
    _user_type: PhantomData<User>,
}

impl<User> EdgeDbStore<User> {
    pub fn new(client: edgedb_tokio::Client) -> Self {
        Self {
            client,
            _user_type: Default::default(),
        }
    }
}

#[async_trait]
impl<UserId, User, Role> UserStore<UserId, Role> for EdgeDbStore<User>
where
    UserId: Eq + Clone + Send + Sync + Hash + ScalarArg + 'static,
    Role: PartialOrd + PartialEq + Clone + Send + Sync + 'static,
    User: AuthUser<UserId, Role> + Queryable,
{
    type User = User;
    async fn load_user(&self, user_id: &UserId) -> Result<Option<Self::User>, Error> {
        let q = "SELECT User {id, password, is_active, is_superuser} FILTER .id = <uuid>$0";
        let user: Option<User> = self.client.query_single(q, &(user_id,)).await?;
        Ok(user)
    }
}
