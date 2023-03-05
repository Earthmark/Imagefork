use protobuf::Message;
use sled::Db;

use crate::protos::Poster;

pub struct Store {
    db: Db,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Sled: {0}")]
    Sled(#[from] sled::Error),
    #[error("Proto: {0}")]
    Proto(#[from] protobuf::Error),
}

type Result<T> = std::result::Result<T, Error>;

impl Store {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        Ok(Self {
            db: sled::open(path)?,
        })
    }

    pub fn get(&self, id: u64) -> Result<Option<Poster>> {
        if let Some(data) = self.db.get(id.to_be_bytes())? {
          Ok(Some(Poster::parse_from_bytes(&data)?))
        } else {
          Ok(None)
        }
    }

    pub fn set(&self, value: &mut Poster) -> Result<()> {
        value.id = self.db.generate_id()?;
        let data = value.write_to_bytes()?;
        self.db.insert(value.id.to_be_bytes(), data.as_slice())?;
        Ok(())
    }
}
