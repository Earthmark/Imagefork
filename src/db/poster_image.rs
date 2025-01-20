use std::io::Write;

use diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    prelude::*,
    serialize::{IsNull, ToSql},
};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

use crate::schema::poster_image::dsl;

use super::DbConn;

#[derive(Clone, Debug, AsExpression, FromSqlRow, Deserialize, Serialize, Default)]
#[diesel(sql_type = crate::schema::sql_types::TextureKind)]
pub enum PosterImageKind {
    #[default]
    Albedo,
    Emissive,
    Normal,
}

impl ToSql<crate::schema::sql_types::TextureKind, Pg> for PosterImageKind {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        match *self {
            PosterImageKind::Albedo => out.write_all(b"albedo")?,
            PosterImageKind::Emissive => out.write_all(b"emissive")?,
            PosterImageKind::Normal => out.write_all(b"normal")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::TextureKind, Pg> for PosterImageKind {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"albedo" => Ok(PosterImageKind::Albedo),
            b"emissive" => Ok(PosterImageKind::Emissive),
            b"normal" => Ok(PosterImageKind::Normal),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Queryable, Selectable, Deserialize, Serialize, Debug)]
#[diesel(table_name = crate::schema::poster_image)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PosterImage {
    poster_id: i64,
    kind: PosterImageKind,
    url: String,
}

impl PosterImage {
    pub async fn get_url(
        db: &mut DbConn,
        poster_id: i64,
        texture: PosterImageKind,
    ) -> crate::Result<Option<String>> {
        Ok(dsl::poster_image
            .find((poster_id, texture))
            .select(dsl::url)
            .get_result(db)
            .await
            .optional()?)
    }
}
