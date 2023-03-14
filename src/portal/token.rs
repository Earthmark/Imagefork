use chrono::{DateTime, Duration, Utc};
use rocket::fairing::{AdHoc, Fairing};
use rocket::http::{Cookie, CookieJar};
use rocket::outcome::{try_outcome, IntoOutcome};
use rocket::request::{FromRequest, Outcome};
use rocket::{Build, Request, Rocket, State};
use rocket_db_pools::Connection;
use serde::{Deserialize, Deserializer, Serialize};

use crate::db::Imagefork;
use crate::db::{Creator, CreatorToken};

fn from_hours<'de, D>(d: D) -> std::result::Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(d).map(Duration::hours)
}

#[derive(Deserialize)]
struct TokenConfig {
    #[serde(deserialize_with = "from_hours")]
    life_limit: Duration,
    #[serde(deserialize_with = "from_hours")]
    refresh_limit: Duration,
}

pub fn fairing() -> impl Fairing {
    AdHoc::try_on_ignite("AuthToken Config", attach_config)
}

async fn attach_config(rocket: Rocket<Build>) -> rocket::fairing::Result {
    match rocket.figment().extract_inner::<TokenConfig>("authToken") {
        Ok(config) => Ok(rocket.manage(config)),
        Err(e) => {
            warn!("Failed to find config: {}", e);
            Err(rocket)
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AuthToken {
    id: i64,
    created: DateTime<Utc>,
}

static TOKEN_COOKIE_NAME: &str = "token";

impl AuthToken {
    pub fn from_cookie_jar(cookie: &CookieJar) -> Option<Self> {
        cookie
            .get_private(TOKEN_COOKIE_NAME)
            .and_then(|cookie| serde_json::from_str::<Self>(cookie.value()).ok())
    }

    pub fn set_in_cookie_jar(id: i64, cookie: &CookieJar) -> Self {
        let created = Utc::now();
        let token = Self { id, created };
        let token_string = serde_json::to_string(&token).unwrap();
        cookie.add_private(Cookie::new(TOKEN_COOKIE_NAME, token_string));
        token
    }

    pub fn remove_from_cookie_jar(cookie: &CookieJar) {
        cookie.remove_private(Cookie::named(TOKEN_COOKIE_NAME));
    }
}

#[async_trait]
impl<'r> FromRequest<'r> for &'r CreatorToken {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user_ref = request
            .local_cache_async(async {
                let mut db = request.guard::<Connection<Imagefork>>().await.succeeded()?;
                let cookies = request.cookies();
                let config = request.guard::<&State<TokenConfig>>().await.succeeded()?;

                if let Some(token) = AuthToken::from_cookie_jar(cookies) {
                    let now = Utc::now();
                    if token.created + config.life_limit < now {
                        AuthToken::remove_from_cookie_jar(cookies);
                        return None;
                    } else if token.created + config.refresh_limit < now {
                        AuthToken::set_in_cookie_jar(token.id, cookies);
                    }
                    Creator::get_token(&mut db, token.id).await.ok().flatten()
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
            Outcome::Success(ModeratorToken(creator))
        } else {
            Outcome::Forward(())
        }
    }
}
