use reqwest::Client;
use rocket::State;
use rocket::fairing::Fairing;
use rocket::http::CookieJar;
use rocket::http::{private::cookie::CookieBuilder, Cookie};
use rocket::response::Redirect;
use rocket_oauth2::{OAuth2, TokenResponse};
use serde::Serialize;

pub struct AuthClient(Client);

impl Default for AuthClient {
    fn default() -> Self {
        Self(
            Client::builder()
                .build()
                .unwrap(),
        )
    }
}

struct GitHub;

pub fn fairing() -> impl Fairing {
    OAuth2::<GitHub>::fairing("github")
}

pub fn routes() -> Vec<rocket::Route> {
    routes![github_login, github_callback, force_login, force_logout]
}

#[get("/force-login/<id>")]
fn force_login(jar: &CookieJar<'_>, id: i64) {
    jar.add_private(CookieBuilder::new("creator_id", id.to_string()).finish());
}

#[get("/login/github")]
fn github_login(oauth2: OAuth2<GitHub>, cookies: &CookieJar<'_>) -> Option<Redirect> {
    oauth2
        .get_redirect(cookies, &["user:email"])
        .map_err(|e| warn!("Error with oauth {}", e))
        .ok()
}

#[derive(Serialize)]
struct GithubTokenRequest<'s> {
    client_id: &'s str,
    client_secret: &'s str,
    code: &'s str,
    redirect_url: &'s str,
}

#[get("/auth/github")]
fn github_callback(c: &State<AuthClient>, token: TokenResponse<GitHub>, cookies: &CookieJar<'_>) -> Redirect {
    token.access_token()
    let token = GithubTokenRequest {
        client_id
    }
    c.0.post("https://github.com/login/oauth/access_token").header("Accept", "application/json").body(body)
    token.access_token();
    Redirect::to("/")
}

#[get("/logout")]
fn force_logout(jar: &CookieJar<'_>) {
    jar.remove_private(Cookie::named("creator_id"));
}
