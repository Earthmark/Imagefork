use rocket::{log::private::warn, response::Redirect, Route, State};
use rocket_db_pools::Connection;

use crate::{
    cache::{Cache, TokenCacheConfig},
    db::{Imagefork, Poster},
};

pub fn routes() -> Vec<Route> {
    routes![handler, ambigous_handler]
}

static SAFE_IMAGE: &str = "canned.webp";
static ERROR_IMAGE: &str = "error.webp";

async fn get_url_of_approx(db: &mut Connection<Imagefork>, width: i32, aspect: f32) -> String {
    match Poster::get_url_of_approx(db, width, aspect).await {
        Ok(Some(url)) => url,
        Ok(None) => SAFE_IMAGE.to_string(),
        Err(e) => {
            warn!("Error getting poster {}", e);
            ERROR_IMAGE.to_string()
        }
    }
}

#[get("/?<width>&<aspect>&<token>")]
async fn handler(
    mut db: Connection<Imagefork>,
    mut cache: Connection<Cache>,
    config: &State<TokenCacheConfig>,
    width: i32,
    aspect: f32,
    token: Option<i64>,
) -> Redirect {
    let url = match token {
        None | Some(0) => get_url_of_approx(&mut db, width, aspect).await,
        Some(token) => Cache::get_or_create(
            &mut cache,
            token,
            config.token_keepalive_minutes,
            get_url_of_approx(&mut db, width, aspect),
        )
        .await
        .unwrap_or_else(|e| {
            warn!("cache error: {}", e);
            ERROR_IMAGE.to_string()
        }),
    };
    Redirect::to(url)
}

#[get("/")]
pub async fn ambigous_handler() -> Redirect {
    Redirect::to(SAFE_IMAGE)
}
