use crate::{
    cache::{CoherencyTokenConn, CoherencyTokenPool},
    db::{DbConn, DbPool, Poster},
    either_resp::EitherResp,
    image::StaticImage,
};
use axum::{
    extract::{FromRef, Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use mediatype::MediaType;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct RedirectConfig {
    pub coherency_token_redis_url: String,
    pub coherency_token_keepalive_minutes: u32,
}

#[derive(Clone)]
struct RedirectSettings {
    redirect_token_keepalive_seconds: u32,
}

#[derive(FromRef, Clone)]
struct RedirectState {
    db: DbPool,
    tokens: CoherencyTokenPool,
    config: RedirectSettings,
}

pub fn create_router(
    db: DbPool,
    tokens: CoherencyTokenPool,
    redirect_token_keepalive_seconds: u32,
) -> Router {
    Router::new()
        .route("/{token}", get(handler))
        .with_state(RedirectState {
            db,
            tokens,
            config: RedirectSettings {
                redirect_token_keepalive_seconds,
            },
        })
}

async fn handle_redirect_internal(
    mut db: DbConn,
    mut cache: CoherencyTokenConn,
    config: RedirectSettings,
    token: Option<&str>,
) -> crate::Result<Option<String>> {
    let id = match token {
        None => Poster::get_id_of_approx(&mut db).await,
        Some(token) => {
            cache
                .get_or_create(
                    token,
                    config.redirect_token_keepalive_seconds,
                    Poster::get_id_of_approx(&mut db),
                )
                .await
        }
    }?;
    if let Some(id) = id {
        Poster::get_url(&mut db, id).await
    } else {
        Ok(None)
    }
}

static WEBP_MEDIA: MediaType =
    MediaType::from_parts(mediatype::names::IMAGE, mediatype::names::WEBP, None, &[]);

static ERROR_IMAGE: StaticImage =
    StaticImage::new(&WEBP_MEDIA, include_bytes!("../images/error.webp"));
static SAFE_IMAGE: StaticImage =
    StaticImage::new(&WEBP_MEDIA, include_bytes!("../images/safe.webp"));

#[axum::debug_handler(state = RedirectState)]
async fn handler(
    db: DbConn,
    cache: CoherencyTokenConn,
    State(config): State<RedirectSettings>,
    Path(token): Path<String>,
) -> EitherResp<Redirect, impl IntoResponse> {
    match handle_redirect_internal(db, cache, config, Some(&token)).await {
        Ok(Some(url)) => EitherResp::A(Redirect::temporary(&url)),
        Ok(None) => EitherResp::B((StatusCode::NOT_FOUND, SAFE_IMAGE.to_response())),
        Err(e) => {
            tracing::warn!("Error resolving redirect {}", e);
            EitherResp::B((StatusCode::INTERNAL_SERVER_ERROR, ERROR_IMAGE.to_response()))
        }
    }
}
