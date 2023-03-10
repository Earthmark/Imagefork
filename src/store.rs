use rocket::{
    fairing::{self, AdHoc, Fairing},
    Build, Rocket,
};
use serde::Deserialize;
use sled::Db;

#[derive(Deserialize)]
struct StoreConfig {
    url: String,
}

pub fn fairing() -> impl Fairing {
    AdHoc::try_on_ignite("Store init", fairing_internal)
}

async fn fairing_internal(rocket: Rocket<Build>) -> fairing::Result {
    let config: StoreConfig = rocket
        .figment()
        .focus("databases")
        .extract_inner::<StoreConfig>("redirects")
        .expect("Sled config");
    Ok(rocket.manage(Store::new(config.url)))
}

pub struct Store {
    db: Db,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Sled: {0}")]
    Sled(#[from] sled::Error),
    #[error("Serde: {0}")]
    Serde(#[from] serde_json::Error),
}

type Result<T> = std::result::Result<T, Error>;

impl Store {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        Ok(Self {
            db: sled::open(path)?,
        })
    }
}
