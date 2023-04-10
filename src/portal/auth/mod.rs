pub mod github;

use crate::db::CreatorToken;
use reqwest::Client;
use reqwest::{
    header,
    header::{HeaderMap, HeaderValue},
};
use rocket::http::CookieJar;
use rocket::response::Redirect;

pub struct AuthClient(Client);

pub fn routes() -> Vec<rocket::Route> {
    routes![logout]
}

impl Default for AuthClient {
    fn default() -> Self {
        let mut headers = HeaderMap::default();
        headers.append(header::ACCEPT, HeaderValue::from_static("application/json"));
        headers.append(
            header::USER_AGENT,
            HeaderValue::from_static("Earthmark-Imagefork"),
        );
        Self(Client::builder().default_headers(headers).build().unwrap())
    }
}

#[get("/logout")]
fn logout(jar: &CookieJar<'_>) -> Redirect {
    CreatorToken::remove_from_cookie_jar(jar);
    Redirect::to("/")
}

#[cfg(test)]
pub mod test {
    use crate::db::{CreatorToken, Imagefork};
    use rocket::http::CookieJar;
    use rocket::response::Redirect;
    use rocket::Route;
    use rocket_db_pools::Connection;

    pub fn routes() -> Vec<Route> {
        routes![force_login]
    }

    #[get("/force-login/<id>")]
    pub async fn force_login(
        mut db: Connection<Imagefork>,
        jar: &CookieJar<'_>,
        id: i64,
    ) -> Result<Redirect, crate::Error> {
        if let Some(token) = CreatorToken::relogin(&mut db, id).await? {
            token.set_in_cookie_jar(jar);
        }
        Ok(Redirect::to("/"))
    }
}
