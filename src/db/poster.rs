use super::Imagefork;
use crate::image_meta::Metadata;
use chrono::NaiveDateTime;
use rocket_db_pools::{sqlx, Connection};
use serde::{Deserialize, Serialize};
use sqlx::Result;

#[derive(Deserialize, Serialize, Debug)]
pub struct Poster {
    id: i64,
    creator: i64,
    creation_time: NaiveDateTime,
    url: String,
    height: i64,
    width: i64,
    hash: String,
    dead_url: bool,
    life_last_checked: NaiveDateTime,
    start_time: NaiveDateTime,
    end_time: Option<NaiveDateTime>,
    stopped: bool,
    lockout: bool,
}

impl Poster {
    pub async fn get(
        db: &mut Connection<Imagefork>,
        poster_id: i64,
        creator_id: i64,
    ) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM Posters WHERE id = ? AND creator = ? LIMIT 1",
            poster_id,
            creator_id
        )
        .fetch_optional(&mut **db)
        .await
    }

    pub async fn get_all_by_creator(db: &mut Connection<Imagefork>, creator_id: i64) -> Result<Vec<Self>> {
        sqlx::query_as!(
          Self,
            "SELECT * FROM Posters WHERE creator = ?",
            creator_id
        )
        .fetch_all(&mut **db)
        .await
    }

    pub async fn post(
        db: &mut Connection<Imagefork>,
        creator_id: i64,
        url: &str,
        metadata: &Metadata,
    ) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "INSERT INTO Posters (Creator, Url, Height, Width, Hash)
            SELECT ?, ?, ?, ?, ?
            WHERE (SELECT COUNT(*) FROM Posters WHERE creator = ?) < (SELECT poster_limit FROM Creators WHERE id = ? LIMIT 1)
            RETURNING *;
            ",
            creator_id,
            url,
            metadata.height,
            metadata.width,
            metadata.hash,
            creator_id,
            creator_id,
        )
        .fetch_optional(&mut **db)
        .await
    }
}
