use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use bb8_redis::{redis::cmd, RedisConnectionManager};
use metrics::counter;
use sha2::{digest::Output, Digest, Sha256};
use std::future::Future;

use crate::Error;

pub type CoherencyTokenManager = RedisConnectionManager;
pub type CoherencyTokenPool = bb8::Pool<CoherencyTokenManager>;

pub struct CoherencyTokenConn(bb8::PooledConnection<'static, CoherencyTokenManager>);

fn hash_token(token: &str) -> Output<Sha256> {
    let mut hasher = Sha256::new();
    hasher.update(token);
    hasher.finalize()
}

impl<S> FromRequestParts<S> for CoherencyTokenConn
where
    S: Send + Sync,
    CoherencyTokenPool: FromRef<S>,
{
    type Rejection = Error;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = CoherencyTokenPool::from_ref(state);

        let conn = pool.get_owned().await?;

        Ok(Self(conn))
    }
}

impl CoherencyTokenConn {
    pub async fn get_or_create(
        &mut self,
        token: &str,
        token_keepalive_seconds: u32,
        init: impl Future<Output = crate::Result<Option<i64>>>,
    ) -> crate::Result<Option<i64>> {
        let hash = hash_token(token);

        if let Some(target) = cmd("GETEX")
            .arg(hash.as_slice())
            .arg("EX")
            .arg(token_keepalive_seconds)
            .query_async(&mut *self.0)
            .await?
        {
            counter!("/coherency_token/hit").increment(1);
            Ok(target)
        } else {
            if let Some(target) = init.await? {
                let try_set: Option<i64> = cmd("SET")
                    .arg(hash.as_slice())
                    .arg(target)
                    .arg("NX")
                    .arg("GET")
                    .arg("EX")
                    .arg(token_keepalive_seconds)
                    .query_async(&mut *self.0)
                    .await?;
                if let Some(target) = try_set {
                    counter!("/coherency_token/update_discarded").increment(1);
                    Ok(Some(target))
                } else {
                    counter!("/coherency_token/update").increment(1);
                    Ok(Some(target))
                }
            } else {
                Ok(None)
            }
        }
    }
}

/*
#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::Cache;
    use crate::test::TestRocket;
    use rocket::serde::json::Json;
    use rocket_db_pools::{deadpool_redis::redis::cmd, Connection};

    async fn echo(value: i64) -> i64 {
        value
    }

    #[get("/test/set?<token>&<value>")]
    async fn set(mut db: Connection<Cache>, token: &str, value: i64) -> Json<i64> {
        Cache::get_or_create(&mut db, token, 1, echo(value))
            .await
            .unwrap()
            .into()
    }

    #[get("/test/get-raw?<token>")]
    async fn get_raw(mut db: Connection<Cache>, token: &str) -> Json<i64> {
        cmd("GET")
            .arg(token)
            .query_async::<_, Option<i64>>(&mut *db)
            .await
            .unwrap()
            .unwrap_or(0)
            .into()
    }

    #[get("/test/force-delete?<token>")]
    async fn force_delete(mut db: Connection<Cache>, token: &str) -> Option<Json<i64>> {
        let hash = super::hash_token(token);
        cmd("GETDEL")
            .arg(hash.as_slice())
            .query_async::<_, Option<i64>>(&mut *db)
            .await
            .unwrap()
            .map(Into::into)
    }

    #[test]
    fn same_key_returns_cached_value() {
        let client = TestRocket::new(routes![set, force_delete]).client();
        client.get(uri!(force_delete(token = "A")));
        client.get(uri!(force_delete(token = "B")));
        assert_eq!(&client.get_string(uri!(set(token = "A", value = 1))), "1");
        assert_eq!(&client.get_string(uri!(set(token = "A", value = 2))), "1");
        assert_eq!(&client.get_string(uri!(set(token = "B", value = 2))), "2");
        assert_eq!(&client.get_string(uri!(set(token = "B", value = 1))), "2");
        client.get(uri!(force_delete(token = "A")));
        client.get(uri!(force_delete(token = "B")));
    }

    #[test]
    fn same_key_ages_out_returns_cached_value() {
        let client = TestRocket::new(routes![set, force_delete]).client();
        client.get(uri!(force_delete(token = "C")));
        assert_eq!(&client.get_string(uri!(set(token = "C", value = 1))), "1");
        std::thread::sleep(Duration::from_millis(1500));
        assert_eq!(&client.get_string(uri!(set(token = "C", value = 2))), "2");
        client.get(uri!(force_delete(token = "C")));
        client.get(uri!(force_delete(token = "C")));
    }

    #[test]
    fn ensure_cache_is_hashed() {
        let client = TestRocket::new(routes![set, force_delete, get_raw]).client();
        client.get(uri!(force_delete(token = "D")));
        assert_eq!(&client.get_string(uri!(set(token = "D", value = 1))), "1");
        assert_eq!(client.get_string(uri!(get_raw(token = "D"))), "0");
        client.get(uri!(force_delete(token = "D")));
    }
}
*/
