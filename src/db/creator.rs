use super::Imagefork;
use chrono::NaiveDateTime;
use rocket_db_pools::{sqlx, Connection};
use serde::{Deserialize, Serialize};
use sqlx::Result;

#[derive(Deserialize, Serialize, Debug)]
pub struct Creator {
    id: i64,
    email: String,
    creation_time: NaiveDateTime,
    referal_token: Option<String>,
    lockout: bool,
    moderator: bool,
    poster_limit: i64,
}

impl Creator {
    pub async fn get(db: &mut Connection<Imagefork>, id: i64) -> Result<Option<Self>> {
        Ok(
            sqlx::query_as!(Self, "SELECT * FROM Creators WHERE id = ?", id)
                .fetch_optional(&mut **db)
                .await?,
        )
    }

    pub async fn can_add_posters(
        db: &mut Connection<Imagefork>,
        creator_id: i64,
    ) -> Result<Option<bool>> {
        struct CanAddPoster {
            can_add: i64,
        }

        Ok(sqlx::query_as!(
            CanAddPoster,
            "SELECT poster_limit > (SELECT COUNT(*) FROM Posters WHERE creator = id) AS can_add
            FROM Creators WHERE id = ?
            LIMIT 1
            ",
            creator_id,
        )
        .fetch_optional(&mut **db)
        .await?
        .map(|c| c.can_add > 0))
    }
}
