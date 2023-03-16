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
    height: i32,
    width: i32,
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
            "SELECT * FROM Posters WHERE id = $1 AND creator = $2 LIMIT 1",
            poster_id,
            creator_id
        )
        .fetch_optional(&mut **db)
        .await
    }

    pub async fn get_all_by_creator(
        db: &mut Connection<Imagefork>,
        creator_id: i64,
    ) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM Posters WHERE creator = $1", creator_id)
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
            SELECT $1, $2, $3, $4, $5
            WHERE (SELECT COUNT(*) FROM Posters WHERE creator = $1) < (SELECT poster_limit FROM Creators WHERE id = $1 LIMIT 1)
            RETURNING *;
            ",
            creator_id,
            url,
            metadata.height as i32,
            metadata.width as i32,
            metadata.hash,
        )
        .fetch_optional(&mut **db)
        .await
    }

    pub async fn get_url_of_approx(
        db: &mut Connection<Imagefork>,
        _width: i32,
        _aspect: f32,
    ) -> Result<Option<String>> {
        struct FoundPoster {
            url: String,
        }
        Ok(sqlx::query_as!(
            FoundPoster,
            "SELECT url FROM Posters
            WHERE id IN (SELECT id FROM Posters ORDER BY RANDOM() LIMIT 1)
            LIMIT 1"
        )
        .fetch_optional(&mut **db)
        .await?
        .map(|f| f.url))
    }
}
