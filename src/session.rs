use std::fmt::Display;

use time::{PrimitiveDateTime, UtcOffset};
use tower_sessions::{
    session::{Id, Record},
    session_store::{self},
    ExpiredDeletion, SessionStore,
};
use tracing::instrument;

use crate::db::DbPool;

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
}

#[async_trait::async_trait]
impl SessionStore for Store {
    #[instrument(skip(self, session_record))]
    async fn create(&self, session_record: &mut Record) -> session_store::Result<()> {
        let exp_time = session_record.expiry_date.to_offset(UtcOffset::UTC);
        let exp_time = PrimitiveDateTime::new(exp_time.date(), exp_time.time());

        while 1
            != sqlx::query!(
                r#"INSERT INTO sessions (id, expiry_date, data)
                VALUES ($1, $2, $3)"#,
                session_record.id.to_string(),
                exp_time,
                rmp_serde::to_vec(&session_record).map_err(to_encode)?
            )
            .execute(&self.db)
            .await
            .map_err(to_backend)?
            .rows_affected()
        {
            session_record.id = Id::default();
        }

        Ok(())
    }

    #[instrument(skip(self, session_record))]
    async fn save(&self, session_record: &Record) -> session_store::Result<()> {
        let exp_time = session_record.expiry_date.to_offset(UtcOffset::UTC);
        let exp_time = PrimitiveDateTime::new(exp_time.date(), exp_time.time());

        sqlx::query!(
            r#"
        UPDATE sessions
        SET expiry_date = $2, data = $3
        WHERE id = $1
        "#,
            session_record.id.to_string(),
            exp_time,
            rmp_serde::to_vec(&session_record).map_err(to_encode)?
        )
        .execute(&self.db)
        .await
        .map_err(to_backend)?;

        Ok(())
    }

    #[instrument(skip(self, session_id))]
    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let record = sqlx::query!(
            r#"
        SELECT data
        FROM sessions
        WHERE id = $1 AND expiry_date > timezone('utc', now())"#,
            session_id.to_string()
        )
        .fetch_optional(&self.db)
        .await
        .map_err(to_backend)?;

        if let Some(record) = record {
            Ok(Some(
                rmp_serde::from_slice(&record.data).map_err(to_decode)?,
            ))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip(self, session_id))]
    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        sqlx::query!(
            r#"
        DELETE
        FROM sessions
        WHERE id = $1"#,
            session_id.to_string()
        )
        .execute(&self.db)
        .await
        .map_err(to_backend)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl ExpiredDeletion for Store {
    async fn delete_expired(&self) -> session_store::Result<()> {
        sqlx::query!(
            r#"
        DELETE
        FROM sessions
        WHERE expiry_date < timezone('utc', now())"#
        )
        .execute(&self.db)
        .await
        .map_err(to_backend)?;

        Ok(())
    }
}
