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
        sqlx::query_as!(Self, "SELECT * FROM Creators WHERE id = ? LIMIT 1", id)
            .fetch_optional(&mut **db)
            .await
    }

    pub async fn get_token(db: &mut Connection<Imagefork>, id: i64) -> Result<Option<CreatorToken>> {
        sqlx::query_as!(
            CreatorToken,
            "SELECT id, lockout, moderator FROM Creators WHERE id = ?",
            id
        )
        .fetch_optional(&mut **db)
        .await
    }

    pub async fn get_or_create_by_email(
        db: &mut Connection<Imagefork>,
        email: &str,
    ) -> Result<i64> {
        struct CreatorId {
            id: i64,
        }
        let id = if let Some(id) = sqlx::query_as!(
            CreatorId,
            "SELECT id FROM Creators WHERE email = ? LIMIT 1",
            email
        )
        .fetch_optional(&mut **db)
        .await?
        {
            id
        } else {
            sqlx::query_as!(
                CreatorId,
                "INSERT INTO Creators (Email)
                SELECT ?
                RETURNING id;
                ",
                email
            )
            .fetch_one(&mut **db)
            .await?
        };
        Ok(id.id)
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
            "SELECT poster_limit > (SELECT COUNT(*) FROM Posters WHERE creator = Creators.id) AS can_add
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

pub struct CreatorToken {
    id: i64,
    lockout: bool,
    moderator: bool,
}

impl CreatorToken {
    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn locked_out(&self) -> bool {
        self.lockout
    }
    pub fn is_moderator(&self) -> bool {
        self.moderator
    }
}
