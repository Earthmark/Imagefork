pub mod creator;
pub mod creator_token;
pub mod poster;

use std::ops::{Deref, DerefMut};

use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};

pub use poster::Poster;

use crate::Error;

pub type DbManager = AsyncDieselConnectionManager<AsyncPgConnection>;
pub type DbPool = bb8::Pool<DbManager>;

pub struct DbConn(bb8::PooledConnection<'static, DbManager>);

#[axum::async_trait]
impl<S> FromRequestParts<S> for DbConn
where
    S: Send + Sync,
    DbPool: FromRef<S>,
{
    type Rejection = Error;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = DbPool::from_ref(state);

        let conn = pool.get_owned().await?;

        Ok(Self(conn))
    }
}

impl Deref for DbConn {
    type Target = bb8::PooledConnection<'static, AsyncDieselConnectionManager<AsyncPgConnection>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DbConn {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
