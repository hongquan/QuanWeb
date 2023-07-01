use std::hash::Hash;
use std::fmt::Debug;
use std::marker::PhantomData;

use async_trait::async_trait;
use axum_login::{UserStore as UserStoreTrait, AuthUser};
use edgedb_protocol::{query_arg::ScalarArg, queryable::Queryable};
use eyre::Error;


#[derive(Clone, Debug)]
pub struct EdgeDbStore<IUser> {
    client: edgedb_tokio::Client,
    _user_type: PhantomData<IUser>,
}

impl<IUser> EdgeDbStore<IUser> {
    pub fn new(client: edgedb_tokio::Client) -> Self {
        Self {
            client,
            _user_type: Default::default(),
        }
    }
}

#[async_trait]
impl<UserId, IUser, Role> UserStoreTrait<UserId, Role> for EdgeDbStore<IUser>
where
    UserId: Eq + Clone + Send + Sync + Hash + Debug + ScalarArg + 'static,
    Role: PartialOrd + PartialEq + Clone + Send + Sync + 'static,
    IUser: AuthUser<UserId, Role> + Queryable,
{
    type User = IUser;
    async fn load_user(&self, user_id: &UserId) -> Result<Option<Self::User>, Error> {
        tracing::info!("To load user with ID {:?}", user_id);
        let q = "SELECT User {id, username, email, password, is_active, is_superuser} FILTER .id = <uuid>$0";
        let user: Option<IUser> = self.client.query_single(q, &(user_id,)).await?;
        Ok(user)
    }
}
