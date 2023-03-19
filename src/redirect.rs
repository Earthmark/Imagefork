use rocket::{log::private::warn, response::Redirect, Route, State};
use rocket_db_pools::Connection;

use crate::{
    cache::{Cache, TokenCacheConfig},
    db::{Imagefork, Poster},
};

pub fn routes() -> Vec<Route> {
    routes![handler]
}

static SAFE_IMAGE: &str = "canned.webp";
static ERROR_IMAGE: &str = "error.webp";

async fn get_url_of_approx(db: &mut Connection<Imagefork>) -> String {
    match Poster::get_url_of_approx(db).await {
        Ok(Some(url)) => url,
        Ok(None) => SAFE_IMAGE.to_string(),
        Err(e) => {
            warn!("Error getting poster {}", e);
            ERROR_IMAGE.to_string()
        }
    }
}

#[get("/<token>")]
async fn handler(
    mut db: Connection<Imagefork>,
    mut cache: Connection<Cache>,
    config: &State<TokenCacheConfig>,
    token: Option<&str>,
) -> Redirect {
    let url = match token {
        None => get_url_of_approx(&mut db).await,
        Some(token) => Cache::get_or_create(
            &mut cache,
            token,
            config.token_keepalive_minutes * 60,
            get_url_of_approx(&mut db),
        )
        .await
        .unwrap_or_else(|e| {
            warn!("cache error: {}", e);
            ERROR_IMAGE.to_string()
        }),
    };
    Redirect::to(url)
}
