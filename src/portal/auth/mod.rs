pub mod github;

use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket_db_pools::Connection;

use crate::db::{CreatorToken, Imagefork};

pub struct AuthClient(Client);

pub fn routes() -> Vec<rocket::Route> {
  routes![force_logout, force_login]
}

impl Default for AuthClient {
    fn default() -> Self {
        let mut headers = HeaderMap::default();
        headers.append("Accept", HeaderValue::from_static("application/json"));
        headers.append(
            "User-Agent",
            HeaderValue::from_static("Earthmark-Imagefork"),
        );
        Self(Client::builder().default_headers(headers).build().unwrap())
    }
}

#[get("/logout")]
fn force_logout(jar: &CookieJar<'_>) -> Redirect {
    CreatorToken::remove_from_cookie_jar(jar);
    Redirect::to("/")
}

#[get("/force-login/<id>")]
async fn force_login(mut db: Connection<Imagefork>, jar: &CookieJar<'_>, id: i64) -> Result<Redirect, crate::Error> {
    if let Some(token) = CreatorToken::relogin(&mut db, id).await? {
        token.set_in_cookie_jar(jar);
    }
    Ok(Redirect::to("/"))
}
