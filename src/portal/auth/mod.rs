pub mod github;

use axum::Router;

//pub const NEXT_URL_KEY: &str = "auth.next-url";

pub fn routes() -> Router {
    Router::new().nest("/github", github::routes())
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
