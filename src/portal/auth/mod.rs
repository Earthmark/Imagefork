pub mod github;

use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use rocket::http::CookieJar;
use rocket::response::Redirect;

use crate::portal::token::AuthToken;

pub struct AuthClient(Client);

pub fn routes() -> Vec<rocket::Route> {
  routes![force_login, force_logout]
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

#[get("/force-login/<id>")]
fn force_login(jar: &CookieJar<'_>, id: i64) -> Redirect {
    AuthToken::set_in_cookie_jar(id, jar);
    Redirect::to("/")
}

#[get("/logout")]
fn force_logout(jar: &CookieJar<'_>) -> Redirect {
    AuthToken::remove_from_cookie_jar(jar);
    Redirect::to("/")
}
