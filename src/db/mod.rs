pub mod creator;
pub mod poster;

use std::ops::{Deref, DerefMut};

use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use diesel_async::{
    pooled_connection::{
        bb8::{Pool, PooledConnection, RunError},
        AsyncDieselConnectionManager,
    },
    AsyncPgConnection,
};

pub use poster::Poster;

use crate::Error;

type DbManager = AsyncDieselConnectionManager<AsyncPgConnection>;
pub type DbPool = Pool<AsyncPgConnection>;

pub struct DbConn(PooledConnection<'static, AsyncPgConnection>);

pub async fn build_pool(url: &str) -> crate::Result<DbPool> {
    Ok(DbPool::builder().build(DbManager::new(url)).await?)
}

impl DbConn {
    pub async fn from_pool(pool: &DbPool) -> Result<DbConn, RunError> {
        Ok(Self(pool.get_owned().await?))
    }
}

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
    type Target = PooledConnection<'static, AsyncPgConnection>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DbConn {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
