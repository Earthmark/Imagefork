use rocket_db_pools::{deadpool_redis, Connection, Database};
use serde::Deserialize;
use std::future::Future;

use deadpool_redis::redis::cmd;

#[derive(Database)]
#[database("tokens")]
pub struct Cache(deadpool_redis::Pool);

#[derive(Deserialize)]
pub struct TokenCacheConfig {
    pub token_keepalive_minutes: i32,
}

impl Cache {
    pub async fn get_or_create(
        db: &mut Connection<Cache>,
        token: i64,
        token_keepalive_minutes: i32,
        init: impl Future<Output = String>,
    ) -> Result<String, crate::Error> {
        if let Some(target) = cmd("GETEX")
            .arg(token)
            .arg("EX")
            .arg(token_keepalive_minutes * 60)
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
                .arg(token_keepalive_minutes * 60)
                .query_async(&mut **db)
                .await?;
            Ok(try_set.unwrap_or(target))
        }
    }
}
