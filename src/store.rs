use sled::Db;

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
