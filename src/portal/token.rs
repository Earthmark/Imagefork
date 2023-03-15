use chrono::{Duration, Utc};
use rocket::fairing::{AdHoc, Fairing};
use rocket::http::{Cookie, CookieJar};
use rocket::outcome::{try_outcome, IntoOutcome};
use rocket::request::{FromRequest, Outcome};
use rocket::{Build, Request, Rocket, State};
use rocket_db_pools::Connection;
use serde::{Deserialize, Deserializer};

use crate::db::CreatorToken;
use crate::db::{Imagefork, LoginKind};

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

static TOKEN_COOKIE_NAME: &str = "token";

impl CreatorToken {
    fn from_cookie_jar(cookie: &CookieJar) -> Option<String> {
        cookie
            .get_private(TOKEN_COOKIE_NAME)
            .and_then(|cookie| serde_json::from_str::<String>(cookie.value()).ok())
    }

    pub fn set_in_cookie_jar(&self, cookie: &CookieJar) {
        cookie.add_private(Cookie::new(TOKEN_COOKIE_NAME, self.token.to_string()));
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

                if let Some(token) = CreatorToken::from_cookie_jar(cookies) {
                    if let Some(token) = CreatorToken::get_by_token(&mut db, &token).await.ok()? {
                        let now = Utc::now();
                        if token.minting_time + config.life_limit < now {
                            CreatorToken::remove_from_cookie_jar(cookies);
                            None
                        } else if token.minting_time + config.refresh_limit < now {
                            CreatorToken::login(&mut db, LoginKind::Id(token.id))
                                .await
                                .ok()
                        } else {
                            Some(token)
                        }
                    } else {
                        None
                    }
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
        if creator.moderator {
            Outcome::Success(ModeratorToken(creator))
        } else {
            Outcome::Forward(())
        }
    }
}
