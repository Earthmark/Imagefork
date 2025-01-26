use std::fmt::Display;

use diesel::{dsl::now, prelude::*};
use diesel_async::RunQueryDsl;
use time::{PrimitiveDateTime, UtcOffset};
use tower_sessions::{
    session::{Id, Record},
    session_store::{self},
    ExpiredDeletion, SessionStore,
};
use tracing::instrument;

use crate::db::{DbConn, DbPool};
use crate::schema::sessions::dsl;

#[derive(Debug, Clone)]
pub struct Store {
    db: DbPool,
}

fn to_backend(err: impl Display) -> session_store::Error {
    session_store::Error::Backend(err.to_string())
}
fn to_encode(err: impl Display) -> session_store::Error {
    session_store::Error::Encode(err.to_string())
}
fn to_decode(err: impl Display) -> session_store::Error {
    session_store::Error::Decode(err.to_string())
}

impl Store {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    async fn db(&self) -> session_store::Result<DbConn> {
        DbConn::from_pool(&self.db).await.map_err(to_backend)
    }
}

#[async_trait::async_trait]
impl SessionStore for Store {
    #[instrument(skip(self, session_record))]
    async fn create(&self, session_record: &mut Record) -> session_store::Result<()> {
        let db = &mut self.db().await?;

        let exp_time = session_record.expiry_date.to_offset(UtcOffset::UTC);
        let exp_time = PrimitiveDateTime::new(exp_time.date(), exp_time.time());

        while 1
            != diesel::insert_into(dsl::sessions)
                .values((
                    dsl::id.eq(session_record.id.to_string()),
                    dsl::expiry_date.eq(exp_time),
                    dsl::data.eq(rmp_serde::to_vec(&session_record).map_err(to_encode)?),
                ))
                .execute(db)
                .await
                .map_err(to_backend)?
        {
            session_record.id = Id::default();
        }

        Ok(())
    }

    #[instrument(skip(self, session_record))]
    async fn save(&self, session_record: &Record) -> session_store::Result<()> {
        let db = &mut self.db().await?;

        let exp_time = session_record.expiry_date.to_offset(UtcOffset::UTC);
        let exp_time = PrimitiveDateTime::new(exp_time.date(), exp_time.time());

        diesel::update(dsl::sessions.find(session_record.id.to_string()))
            .set((
                dsl::expiry_date.eq(exp_time),
                dsl::data.eq(rmp_serde::to_vec(&session_record).map_err(to_encode)?),
            ))
            .execute(db)
            .await
            .map_err(to_backend)?;

        Ok(())
    }

    #[instrument(skip(self, session_id))]
    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let db = &mut self.db().await?;

        let data: Option<Vec<u8>> = dsl::sessions
            .filter(dsl::expiry_date.gt(now))
            .find(session_id.to_string())
            .select(dsl::data)
            .get_result(db)
            .await
            .optional()
            .map_err(to_backend)?;

        if let Some(data) = data {
            Ok(Some(rmp_serde::from_slice(&data).map_err(to_decode)?))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip(self, session_id))]
    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        let db = &mut self.db().await?;

        diesel::delete(dsl::sessions.find(session_id.to_string()))
            .execute(db)
            .await
            .map_err(to_backend)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl ExpiredDeletion for Store {
    async fn delete_expired(&self) -> session_store::Result<()> {
        let db = &mut self.db().await?;
        diesel::delete(dsl::sessions)
            .filter(dsl::expiry_date.le(now))
            .execute(db)
            .await
            .map_err(to_backend)?;

        Ok(())
    }
}
