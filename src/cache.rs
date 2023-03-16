use rocket_db_pools::{deadpool_redis, Connection, Database};
use std::future::Future;

use deadpool_redis::redis::cmd;

#[derive(Database)]
#[database("tokens")]
pub struct Cache(deadpool_redis::Pool);

impl Cache {
    pub async fn get_or_create(
        db: &mut Connection<Cache>,
        token: i64,
        init: impl Future<Output = String>,
    ) -> Result<String, crate::Error> {
        if let Some(target) = cmd("GETEX")
            .arg(token)
            .arg("EX")
            .arg(500)
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
                .arg(500)
                .query_async(&mut **db)
                .await?;
            Ok(try_set.unwrap_or(target))
        }
    }
}
