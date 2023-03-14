use std::{future::Future, time::Duration};

pub struct Cache(moka::future::Cache<i64, String>);

impl Default for Cache {
    fn default() -> Self {
        let db = moka::future::Cache::builder()
            .time_to_live(Duration::from_secs(60 * 60 * 24 * 3))
            .time_to_idle(Duration::from_secs(60 * 60 * 24))
            .build();
        Self(db)
    }
}

impl Cache {
    pub async fn get_or_create(&self, token: i64, init: impl Future<Output = String>) -> String {
        self.0.get_with(token, init).await
    }
}
