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
    pub minting_time: DateTime<Utc>,
    pub moderator: bool,
    pub lockout: bool,
}

struct OptionCreatorToken {
    id: i64,
    token: Option<String>,
    minting_time: Option<NaiveDateTime>,
    moderator: bool,
    lockout: bool,
}

pub enum LoginKind<'a> {
    Id(i64),
    Email(&'a str),
}

fn generate_token() -> String {
    let mut token = [0; 32];
    rand::thread_rng().try_fill_bytes(&mut token).unwrap();
    base64::engine::general_purpose::URL_SAFE.encode(token)
}

impl CreatorToken {
    fn from_option(token: OptionCreatorToken) -> Option<Self> {
        match token {
            OptionCreatorToken {
                id,
                moderator,
                lockout,
                token: Some(token),
                minting_time: Some(minting_time),
            } => Some(CreatorToken {
                id,
                moderator,
                lockout,
                token,
                minting_time: DateTime::from_utc(minting_time, Utc),
            }),
            _ => None,
        }
    }

    pub async fn get_by_token(db: &mut Connection<Imagefork>, token: &str) -> Result<Option<Self>> {
        let token = sqlx::query_as!(
            OptionCreatorToken,
            "SELECT id, token, minting_time, moderator, lockout
            FROM Creators WHERE token = ? LIMIT 1",
            token
        )
        .fetch_optional(&mut **db)
        .await?;

        Ok(token.and_then(CreatorToken::from_option))
    }

    pub async fn login(db: &mut Connection<Imagefork>, login: LoginKind<'_>) -> Result<Self> {
        let token = generate_token();

        let now = Utc::now().naive_utc();

        Ok(match login {
            LoginKind::Id(id) => {
                sqlx::query_as!(
                    OptionCreatorToken,
                    "UPDATE Creators
                    SET token = ?, minting_time = ?
                    WHERE id = ?
                    RETURNING id, token, minting_time, moderator, lockout",
                    token,
                    now,
                    id
                )
                .fetch_optional(&mut **db)
                .await?
            }
            LoginKind::Email(email) => {
                sqlx::query_as!(
                    OptionCreatorToken,
                    "INSERT OR IGNORE INTO Creators (Email) VALUES (?);
                    UPDATE Creators
                    SET token = ?, minting_time = ?
                    WHERE email = ?
                    RETURNING id, token, minting_time, moderator, lockout",
                    email,
                    token,
                    now,
                    email
                )
                .fetch_optional(&mut **db)
                .await?
            }
        }
        .and_then(CreatorToken::from_option)
        .expect("failed to insert creation token"))
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Creator {
    id: i64,
    email: String,
    creation_time: NaiveDateTime,
    lockout: bool,
    moderator: bool,
    poster_limit: i64,
}

impl Creator {
    pub async fn get(db: &mut Connection<Imagefork>, id: i64) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, email, creation_time, lockout, moderator, poster_limit
            FROM Creators WHERE id = ? LIMIT 1",
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
