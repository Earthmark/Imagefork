use crate::{
    cache::{CoherencyTokenConn, CoherencyTokenPool},
    db::{
        poster_image::{PosterImage, PosterImageKind},
        DbPool, Poster,
    },
    either_resp::EitherResp,
    image::StaticImage,
};
use axum::{
    extract::{FromRef, Path, Query, State},
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use serde::{Deserialize, Deserializer};

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
    db: &DbPool,
    mut cache: CoherencyTokenConn,
    config: RedirectSettings,
    token: Option<&str>,
    image_kind: PosterImageKind,
) -> crate::Result<Option<String>> {
    let id = match token {
        None => Poster::get_id_of_approx(db).await,
        Some(token) => {
            cache
                .get_or_create(
                    token,
                    config.redirect_token_keepalive_seconds,
                    Poster::get_id_of_approx(db),
                )
                .await
        }
    }?;
    if let Some(id) = id {
        PosterImage::get_url(db, id, image_kind).await
    } else {
        Ok(None)
    }
}

static IMAGE_WEBP: mediatype::MediaType =
    mediatype::MediaType::from_parts(mediatype::names::IMAGE, mediatype::names::WEBP, None, &[]);
static IMAGE_PNG: mediatype::MediaType =
    mediatype::MediaType::from_parts(mediatype::names::IMAGE, mediatype::names::PNG, None, &[]);

static ERROR_IMAGE: StaticImage =
    StaticImage::new(&IMAGE_WEBP, include_bytes!("../images/error.webp"));
static SAFE_IMAGE: StaticImage =
    StaticImage::new(&IMAGE_WEBP, include_bytes!("../images/safe.webp"));

static DEFAULT_NORMAL_PIXEL: StaticImage = StaticImage::new(
    &IMAGE_PNG,
    include_bytes!("../images/default_normal_pixel.png"),
);
static BLACK_PIXEL: StaticImage =
    StaticImage::new(&IMAGE_PNG, include_bytes!("../images/black_pixel.png"));

fn image_fallback(requested_image_kind: PosterImageKind, error: bool) -> impl IntoResponse {
    match requested_image_kind {
        PosterImageKind::Albedo => {
            if error {
                ERROR_IMAGE.clone()
            } else {
                SAFE_IMAGE.clone()
            }
        }
        PosterImageKind::Emissive => BLACK_PIXEL.clone(),
        PosterImageKind::Normal => DEFAULT_NORMAL_PIXEL.clone(),
    }
}

fn image_kind_deserializer<'de, D>(d: D) -> Result<PosterImageKind, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <&str>::deserialize(d)?;
    match s {
        "a" => Ok(PosterImageKind::Albedo),
        "n" => Ok(PosterImageKind::Normal),
        "e" => Ok(PosterImageKind::Emissive),
        _ => Err(serde::de::Error::custom("Unable to parse image kind.")),
    }
}

#[derive(Deserialize)]
struct QueryArgs {
    #[serde(default, rename = "k", deserialize_with = "image_kind_deserializer")]
    image_kind: PosterImageKind,
}

#[axum::debug_handler(state = RedirectState)]
async fn handler(
    cache: CoherencyTokenConn,
    State(db): State<DbPool>,
    State(config): State<RedirectSettings>,
    Path(token): Path<String>,
    Query(QueryArgs { image_kind }): Query<QueryArgs>,
) -> impl IntoResponse {
    match handle_redirect_internal(&db, cache, config, Some(&token), image_kind.clone()).await {
        Ok(Some(url)) => EitherResp::A(Redirect::to(&url)),
        Ok(None) => EitherResp::B(image_fallback(image_kind, false)),
        Err(e) => {
            tracing::warn!("Error resolving redirect {}", e);
            EitherResp::B(image_fallback(image_kind, true))
        }
    }
}
