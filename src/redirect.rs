use crate::{
    cache::{Cache, TokenCacheConfig},
    db::{Imagefork, Poster},
    image::Image,
};
use rocket::{http::MediaType, log::private::warn, response::Redirect, Either, Route, State};
use rocket_db_pools::Connection;
use thiserror::Error;

pub fn routes() -> Vec<Route> {
    routes![handler]
}

#[derive(Error, Debug)]
enum Error {
    #[error("{0}")]
    Actual(#[from] crate::Error),
    #[error("cached error")]
    Cached,
}

async fn handle_redirect_internal(
    mut db: Connection<Imagefork>,
    mut cache: Connection<Cache>,
    config: &State<TokenCacheConfig>,
    token: Option<&str>,
) -> Result<Option<String>, Error> {
    let id = match token {
        None => get_id_of_approx(&mut db).await,
        Some(token) => {
            Cache::get_or_create(
                &mut cache,
                token,
                config.token_keepalive_minutes * 60,
                get_id_of_approx(&mut db),
            )
            .await?
        }
    };
    match id {
        0 => Err(Error::Cached),
        1 => Ok(None),
        id => Poster::get_url(&mut db, id)
            .await
            .map_err(|e| Error::Actual(e.into())),
    }
}

// Defined in the initial setup.
static ERROR_ID: i64 = 0;
static SAFE_ID: i64 = 1;

async fn get_id_of_approx(db: &mut Connection<Imagefork>) -> i64 {
    match Poster::get_id_of_approx(db).await {
        Ok(Some(id)) => id,
        Ok(None) => SAFE_ID,
        Err(e) => {
            warn!("Error getting poster {}", e);
            ERROR_ID
        }
    }
}

static ERROR_IMAGE: Image = Image::new(MediaType::WEBP, include_bytes!("error.webp"));
static SAFE_IMAGE: Image = Image::new(MediaType::WEBP, include_bytes!("safe.webp"));

#[get("/<token>")]
async fn handler(
    db: Connection<Imagefork>,
    cache: Connection<Cache>,
    config: &State<TokenCacheConfig>,
    token: Option<&str>,
) -> Either<Redirect, Image> {
    match handle_redirect_internal(db, cache, config, token).await {
        Ok(Some(url)) => Either::Left(Redirect::found(url)),
        Ok(None) => Either::Right(SAFE_IMAGE.clone()),
        Err(e) => {
            if let Error::Actual(e) = e {
                warn!("Error resolving redirect {}", e);
            }
            Either::Right(ERROR_IMAGE.clone())
        }
    }
}
