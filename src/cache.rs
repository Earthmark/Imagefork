use crate::config::ConfigInfo;
use deadpool_redis::redis::cmd;
use rocket_db_pools::{deadpool_redis, Connection, Database};
use serde::Deserialize;
use sha2::{digest::Output, Digest, Sha256};
use std::future::Future;

#[derive(Database)]
#[database("tokens")]
pub struct Cache(deadpool_redis::Pool);

#[derive(Deserialize)]
pub struct TokenCacheConfig {
    pub token_keepalive_minutes: i32,
}

impl ConfigInfo for TokenCacheConfig {
    fn field() -> &'static str {
        "tokens"
    }

    fn name() -> &'static str {
        "Config for tokens redis db"
    }
}

fn hash_token(token: &str) -> Output<Sha256> {
    let mut hasher = Sha256::new();
    hasher.update(token);
    hasher.finalize()
}

impl Cache {
    pub async fn get_or_create(
        db: &mut Connection<Self>,
        token: &str,
        token_keepalive_seconds: i32,
        init: impl Future<Output = i64>,
    ) -> Result<i64, crate::Error> {
        let hash = hash_token(token);

        if let Some(target) = cmd("GETEX")
            .arg(hash.as_slice())
            .arg("EX")
            .arg(token_keepalive_seconds)
            .query_async(&mut **db)
            .await?
        {
            Ok(target)
        } else {
            let target = init.await;
            let try_set: Option<i64> = cmd("SET")
                .arg(hash.as_slice())
                .arg(target)
                .arg("NX")
                .arg("GET")
                .arg("EX")
                .arg(token_keepalive_seconds)
                .query_async(&mut **db)
                .await?;
            Ok(try_set.unwrap_or(target))
        }
    }
}

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
        assert_eq!(client.get_string(uri!(get_raw(token = "D"))), "");
        client.get(uri!(force_delete(token = "D")));
    }
}
