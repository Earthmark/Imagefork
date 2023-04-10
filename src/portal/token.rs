use chrono::{Duration, Utc};
use rocket::http::{Cookie, CookieJar};
use rocket::outcome::{try_outcome, IntoOutcome};
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State};
use rocket_db_pools::Connection;
use serde::{Deserialize, Deserializer};

use crate::config::ConfigInfo;
use crate::db::CreatorToken;
use crate::db::Imagefork;

fn from_hours<'de, D>(d: D) -> std::result::Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(d).map(Duration::hours)
}

#[derive(Deserialize)]
pub struct TokenConfig {
    #[serde(deserialize_with = "from_hours")]
    life_limit: Duration,
    #[serde(deserialize_with = "from_hours")]
    refresh_limit: Duration,
}

impl ConfigInfo for TokenConfig {
    fn field() -> &'static str {
        "authToken"
    }

    fn name() -> &'static str {
        "Config for auth token"
    }
}

static TOKEN_COOKIE_NAME: &str = "token";

impl CreatorToken {
    fn from_cookie_jar(cookie: &CookieJar) -> Option<String> {
        cookie
            .get_private(TOKEN_COOKIE_NAME)
            .map(|cookie| cookie.value().to_string())
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
    type Error = crate::Error;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user_ref = request
            .local_cache_async(async {
                let mut db = request.guard::<Connection<Imagefork>>().await.succeeded()?;
                let cookies = request.cookies();
                let config = request.guard::<&State<TokenConfig>>().await.succeeded()?;

                if let Some(token) = CreatorToken::from_cookie_jar(cookies) {
                    if let Some(token) = CreatorToken::get_by_token(&mut db, &token).await.ok()? {
                        let now = Utc::now();
                        if token.minting_time() + config.life_limit < now {
                            CreatorToken::remove_from_cookie_jar(cookies);
                            None
                        } else if token.minting_time() + config.refresh_limit < now {
                            CreatorToken::relogin(&mut db, token.id)
                                .await
                                .ok()
                                .flatten()
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
        user_ref
            .as_ref()
            .into_outcome(crate::Error::NotLoggedIn.with_status())
    }
}

pub struct ModeratorToken<'a>(pub &'a CreatorToken);

#[async_trait]
impl<'r> FromRequest<'r> for ModeratorToken<'r> {
    type Error = crate::Error;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let creator = try_outcome!(request.guard::<&CreatorToken>().await);
        if creator.moderator {
            Outcome::Success(ModeratorToken(creator))
        } else {
            Outcome::Failure(crate::Error::UserNotAdmin.with_status())
        }
    }
}

#[cfg(test)]
mod test {
    use super::{CreatorToken, ModeratorToken};
    use crate::test::*;
    use rocket::http::StatusClass;

    #[get("/test/creator-only")]
    async fn creator_only(_auth: &CreatorToken) {}

    #[get("/test/admin-only")]
    async fn admin_only(_auth: ModeratorToken<'_>) {}

    #[test]
    fn creator_only_request_rejected() {
        let client = TestRocket::new(routes![creator_only]).client();
        assert_eq!(
            client.get(uri!(creator_only)).class(),
            StatusClass::ClientError
        );
    }

    #[test]
    fn creator_only_request_accepted() {
        let client = TestRocket::new(routes![creator_only]).client();
        let user = client.creator("pt1");
        user.login();
        assert_eq!(client.get(uri!(creator_only)).class(), StatusClass::Success);
    }

    #[test]
    fn creator_only_request_user_delete_rejected() {
        let client = TestRocket::new(routes![creator_only]).client();
        let user = client.creator("pt2");
        user.login();
        user.delete();
        assert_eq!(
            client.get(uri!(creator_only)).class(),
            StatusClass::ClientError
        );
    }

    #[test]
    fn admin_only_request_rejected() {
        let client = TestRocket::new(routes![admin_only]).client();
        assert_eq!(
            client.get(uri!(admin_only)).class(),
            StatusClass::ClientError
        );
    }

    #[test]
    fn admin_only_request_by_creator_rejected() {
        let client = TestRocket::new(routes![admin_only]).client();
        let user = client.creator("pt3");
        user.login();
        assert_eq!(
            client.get(uri!(admin_only)).class(),
            StatusClass::ClientError
        );
    }

    #[test]
    fn admin_only_request_accepted() {
        let client = TestRocket::new(routes![admin_only]).client();
        let user = client.creator("pt4");
        user.login();
        user.promote();
        assert_eq!(client.get(uri!(admin_only)).class(), StatusClass::Success);
    }

    #[test]
    fn admin_only_request_user_deleted_rejected() {
        let client = TestRocket::new(routes![admin_only]).client();
        let user = client.creator("pt5");
        user.login();
        user.promote();
        user.delete();
        assert_eq!(
            client.get(uri!(admin_only)).class(),
            StatusClass::ClientError
        );
    }
}
