use rocket::outcome::{try_outcome, IntoOutcome};
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use rocket_db_pools::{sqlx, Connection};

use super::Result;
use crate::db::Imagefork;

#[derive(Clone)]
pub struct CreatorToken {
    pub id: i64,
    pub lockout: bool,
    moderator: bool,
}

impl CreatorToken {
    pub async fn get(db: &mut Connection<Imagefork>, id: i64) -> Result<Option<Self>> {
        Ok(sqlx::query_as!(
            Self,
            "SELECT id, lockout, moderator FROM Creators WHERE id = ?",
            id
        )
        .fetch_optional(&mut **db)
        .await?)
    }

    fn is_moderator(&self) -> bool {
        self.moderator
    }
}

#[async_trait]
impl<'r> FromRequest<'r> for &'r CreatorToken {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user_ref = request
            .local_cache_async(async {
                let mut db = request.guard::<Connection<Imagefork>>().await.succeeded()?;
                if let Some(id) = request
                    .cookies()
                    .get_private("creator_id")
                    .and_then(|cookie| cookie.value().parse().ok())
                {
                    CreatorToken::get(&mut db, id).await.ok().flatten()
                } else {
                    None
                }
            })
            .await;
        user_ref.as_ref().or_forward(())
    }
}

pub struct ModeratorToken<'a>(pub &'a CreatorToken);

#[async_trait]
impl<'r> FromRequest<'r> for ModeratorToken<'r> {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let creator = try_outcome!(request.guard::<&CreatorToken>().await);
        if creator.is_moderator() {
            Outcome::Success(ModeratorToken(&creator))
        } else {
            Outcome::Forward(())
        }
    }
}
