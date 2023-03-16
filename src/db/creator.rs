use super::Imagefork;
use base64::Engine;
use chrono::{DateTime, NaiveDateTime, Utc};
use rand::RngCore;
use rocket_db_pools::{sqlx, Connection};
use serde::{Deserialize, Serialize};
use sqlx::Result;

pub struct CreatorToken {
    pub id: i64,
    pub token: String,
    minting_time: NaiveDateTime,
    pub moderator: bool,
    pub lockout: bool,
}

fn generate_token() -> String {
    let mut token = [0; 32];
    rand::thread_rng().try_fill_bytes(&mut token).unwrap();
    base64::engine::general_purpose::URL_SAFE.encode(token)
}

impl CreatorToken {
    pub fn minting_time(&self) -> DateTime<Utc> {
        DateTime::from_utc(self.minting_time, Utc)
    }

    pub async fn get_by_token(db: &mut Connection<Imagefork>, token: &str) -> Result<Option<Self>> {
        let token = sqlx::query_as!(
            Self,
            r#"SELECT id, token AS "token!", minting_time AS "minting_time!", moderator, lockout
            FROM Creators WHERE token = $1 LIMIT 1"#,
            token
        )
        .fetch_optional(&mut **db)
        .await?;

        Ok(token)
    }

    pub async fn relogin(db: &mut Connection<Imagefork>, id: i64) -> Result<Self> {
        let token = generate_token();

        Ok(sqlx::query_as!(
            Self,
            r#"UPDATE Creators
            SET token = $1, minting_time = (now() at time zone 'utc')
            WHERE id = $2
            RETURNING id, token AS "token!", minting_time AS "minting_time!", moderator, lockout"#,
            token,
            id
        )
        .fetch_optional(&mut **db)
        .await?
        .expect("failed to insert auth token"))
    }

    pub async fn login(db: &mut Connection<Imagefork>, email: &str) -> Result<Self> {
        let token = generate_token();

        Ok(sqlx::query_as!(
            Self,
            r#"INSERT INTO Creators (email, token, minting_time) VALUES ($1, $2, (now() at time zone 'utc'))
            ON CONFLICT (email)
            DO UPDATE SET token = $2, minting_time = (now() at time zone 'utc')
            RETURNING id, token AS "token!", minting_time AS "minting_time!", moderator, lockout"#,
            email,
            token,
        )
        .fetch_one(&mut **db)
        .await?)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Creator {
    id: i64,
    email: String,
    creation_time: NaiveDateTime,
    lockout: bool,
    moderator: bool,
    poster_limit: i32,
}

impl Creator {
    pub async fn get(db: &mut Connection<Imagefork>, id: i64) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, email, creation_time, lockout, moderator, poster_limit
            FROM Creators WHERE id = $1 LIMIT 1",
            id
        )
        .fetch_optional(&mut **db)
        .await
    }

    pub async fn can_add_posters(
        db: &mut Connection<Imagefork>,
        creator_id: i64,
    ) -> Result<Option<bool>> {
        struct CanAddPoster {
            can_add: bool,
        }

        Ok(sqlx::query_as!(
            CanAddPoster,
            r#"SELECT poster_limit > (SELECT COUNT(*) FROM Posters WHERE creator = Creators.id) AS "can_add!"
            FROM Creators WHERE id = $1
            "#,
            creator_id,
        )
        .fetch_optional(&mut **db)
        .await?
        .map(|c| c.can_add))
    }
}
