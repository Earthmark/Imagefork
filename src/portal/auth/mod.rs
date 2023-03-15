pub mod github;

use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use rocket::http::CookieJar;
use rocket::response::Redirect;

use crate::db::CreatorToken;

pub struct AuthClient(Client);

pub fn routes() -> Vec<rocket::Route> {
  routes![force_logout]
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
