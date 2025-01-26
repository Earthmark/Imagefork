use serde::{Deserialize, Serialize};
use tracing::instrument;

use super::DbPool;

#[derive(sqlx::Type, Clone, Debug, Deserialize, Serialize, Default)]
#[sqlx(type_name = "texture_kind", rename_all = "lowercase")]
pub enum PosterImageKind {
    #[default]
    Albedo,
    Emissive,
    Normal,
}

pub struct PosterImage {}

impl PosterImage {
    #[instrument(skip(db))]
    pub async fn get_url(
        db: &DbPool,
        poster_id: i64,
        texture: PosterImageKind,
    ) -> crate::Result<Option<String>> {
        Ok(sqlx::query!(
            r#"
        SELECT url
        FROM poster_image
        WHERE poster = $1 AND kind = $2
        "#,
            poster_id,
            texture as _,
        )
        .fetch_optional(db)
        .await?
        .map(|r| r.url))
    }
}
