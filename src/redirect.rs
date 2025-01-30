use crate::{
    cache::{CoherencyTokenConn, CoherencyTokenPool},
    db::{
        poster_image::{PosterImage, PosterImageKind},
        DbPool, Poster,
    },
    image::StaticImage,
};
use axum::{
    extract::{FromRef, Path, Query, State},
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use axum_extra::either::Either;
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
    token: &str,
    image_kind: PosterImageKind,
) -> crate::Result<Option<String>> {
    let id = cache
        .get_or_create(
            token,
            config.redirect_token_keepalive_seconds,
            Poster::get_id_of_approx(db),
        )
        .await?;
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

struct CannedImage {
    albedo: StaticImage,
    emissive: Option<StaticImage>,
    normal: Option<StaticImage>,
}

static DEFAULT_NORMAL_PIXEL: StaticImage = StaticImage::new(
    &IMAGE_PNG,
    include_bytes!("../images/default_normal_pixel.png"),
);

static BLACK_PIXEL: StaticImage =
    StaticImage::new(&IMAGE_PNG, include_bytes!("../images/black_pixel.png"));

impl CannedImage {
    fn image(&self, kind: PosterImageKind) -> StaticImage {
        match kind {
            PosterImageKind::Albedo => &self.albedo,
            PosterImageKind::Emissive => self.emissive.as_ref().unwrap_or(&BLACK_PIXEL),
            PosterImageKind::Normal => self.normal.as_ref().unwrap_or(&DEFAULT_NORMAL_PIXEL),
        }
        .clone()
    }
}

static YOTE_CANNED: CannedImage = CannedImage {
    albedo: StaticImage::new(&IMAGE_WEBP, include_bytes!("../images/yote_albedo.webp")),
    emissive: Some(StaticImage::new(
        &IMAGE_WEBP,
        include_bytes!("../images/yote_emissive.webp"),
    )),
    normal: Some(StaticImage::new(
        &IMAGE_WEBP,
        include_bytes!("../images/yote_normal.webp"),
    )),
};

static ERROR_CANNED: CannedImage = CannedImage {
    albedo: StaticImage::new(&IMAGE_WEBP, include_bytes!("../images/error.webp")),
    emissive: None,
    normal: None,
};

static FALLBACK_IMAGE: CannedImage = CannedImage {
    albedo: StaticImage::new(&IMAGE_WEBP, include_bytes!("../images/safe.webp")),
    emissive: None,
    normal: None,
};

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
    if token == "yote" {
        return Either::E2(YOTE_CANNED.image(image_kind));
    }

    match handle_redirect_internal(&db, cache, config, &token, image_kind.clone()).await {
        Ok(Some(url)) => Either::E1(Redirect::to(&url)),
        Ok(None) => Either::E2(FALLBACK_IMAGE.image(image_kind)),
        Err(e) => {
            tracing::warn!("Error resolving redirect {}", e);
            Either::E2(ERROR_CANNED.image(image_kind))
        }
    }
}
