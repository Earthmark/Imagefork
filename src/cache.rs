use rocket_db_pools::{deadpool_redis, Connection, Database};
use serde::Deserialize;
use std::future::Future;

use deadpool_redis::redis::cmd;

use crate::config::ConfigInfo;

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

impl Cache {
    pub async fn get_or_create(
        db: &mut Connection<Self>,
        token: i64,
        token_keepalive_seconds: i32,
        init: impl Future<Output = String>,
    ) -> Result<String, crate::Error> {
        if let Some(target) = cmd("GETEX")
            .arg(token)
            .arg("EX")
            .arg(token_keepalive_seconds)
            .query_async(&mut **db)
            .await?
        {
            Ok(target)
        } else {
            let target = init.await;
            let try_set: Option<String> = cmd("SET")
                .arg(token)
                .arg(&target)
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
    use rocket_db_pools::{deadpool_redis::redis::cmd, Connection};

    async fn echo(value: String) -> String {
        value
    }

    #[get("/test/set?<token>&<value>&<keepalive>")]
    async fn set(mut db: Connection<Cache>, token: i64, value: String, keepalive: i32) -> String {
        Cache::get_or_create(&mut db, token, keepalive, echo(value))
            .await
            .unwrap()
    }

    #[get("/test/force-delete?<token>")]
    async fn force_delete(mut db: Connection<Cache>, token: i64) -> Option<String> {
        cmd("GETDEL")
            .arg(token)
            .query_async(&mut *db)
            .await
            .unwrap()
    }

    #[test]
    fn same_key_returns_cached_value() {
        let client = TestRocket::new(routes![set, force_delete]).client();
        client.get(uri!(force_delete(token = 4)));
        client.get(uri!(force_delete(token = 3)));
        assert_eq!(
            client.get_string(uri!(set(token = 4, value = "tacos", keepalive = 1))),
            "tacos"
        );
        assert_eq!(
            client.get_string(uri!(set(token = 4, value = "nana", keepalive = 1))),
            "tacos"
        );
        assert_eq!(
            client.get_string(uri!(set(token = 3, value = "nana", keepalive = 1))),
            "nana"
        );
        assert_eq!(
            client.get_string(uri!(set(token = 3, value = "tacos", keepalive = 1))),
            "nana"
        );
        client.get(uri!(force_delete(token = 4)));
        client.get(uri!(force_delete(token = 3)));
    }

    #[test]
    fn same_key_ages_out_returns_cached_value() {
        let client = TestRocket::new(routes![set, force_delete]).client();
        client.get(uri!(force_delete(token = 4)));
        client.get(uri!(force_delete(token = 3)));
        assert_eq!(
            client.get_string(uri!(set(token = 4, value = "tacos", keepalive = 1))),
            "tacos"
        );
        std::thread::sleep(Duration::from_millis(1500));
        assert_eq!(
            client.get_string(uri!(set(token = 4, value = "nana", keepalive = 1))),
            "nana"
        );
        client.get(uri!(force_delete(token = 4)));
        client.get(uri!(force_delete(token = 3)));
    }
}
