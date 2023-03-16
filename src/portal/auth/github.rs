use reqwest::{StatusCode, Url};
use rocket::fairing::Fairing;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::State;
use rocket_db_pools::Connection;
use rocket_oauth2::{OAuth2, TokenResponse};
use serde::Deserialize;

use super::AuthClient;
use crate::db::{CreatorToken, Imagefork};

use crate::Result;

struct GitHub;

pub fn fairing() -> impl Fairing {
    OAuth2::<GitHub>::fairing("github")
}

pub fn routes() -> Vec<rocket::Route> {
    routes![github_login, github_callback]
}

#[get("/login/github")]
fn github_login(oauth2: OAuth2<GitHub>, cookies: &CookieJar<'_>) -> Option<Redirect> {
    oauth2
        .get_redirect(cookies, &["user:email"])
        .map_err(|e| warn!("Error with oauth {}", e))
        .ok()
}

#[get("/auth/github")]
async fn github_callback(
    c: &State<AuthClient>,
    token: TokenResponse<GitHub>,
    mut db: Connection<Imagefork>,
    jar: &CookieJar<'_>,
) -> Result<Redirect> {
    let mut url = Url::parse("https://api.github.com/user/emails").unwrap();
    url.query_pairs_mut()
        .append_pair("access_token", token.access_token());

    #[derive(Deserialize)]
    struct EmailRecord {
        email: String,
        primary: bool,
    }
    let response =
        c.0.get(url)
            .header("Authorization", format!("Bearer {}", token.access_token()))
            .send()
            .await?;
    if response.status() != StatusCode::OK {
        return Err(crate::Error::SystemError(format!(
            "Invalid status code: {}: {}",
            response.status(),
            response.text().await?
        )));
    }
    let emails: Vec<EmailRecord> = response.json().await?;
    let primary_email = emails
        .into_iter()
        .find(|e| e.primary)
        .map(|e| e.email)
        .ok_or(crate::Error::SystemError(
            "Oauth did not resolve to a primary email".to_string(),
        ))?;

    let token = CreatorToken::login(&mut db, &primary_email).await?;

    token.set_in_cookie_jar(jar);

    Ok(Redirect::to("/"))
}
