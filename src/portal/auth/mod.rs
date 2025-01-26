pub mod github;

use axum::{
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use serde::Deserialize;

use crate::auth::AuthSession;

pub const NEXT_URL_KEY: &str = "auth.next-url";

#[derive(Deserialize)]
struct Next {
    next: Option<String>,
}

pub fn routes() -> Router {
    Router::new()
        .route("/logout", get(logout))
        .nest("/github", github::routes())
}

#[axum::debug_handler]
async fn logout(mut auth_session: AuthSession) -> crate::Result<impl IntoResponse> {
    if auth_session.user.is_some() {
        auth_session.logout().await.map_err(std::sync::Arc::new)?;
    }

    Ok(Redirect::to("/"))
}

/*
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
*/
