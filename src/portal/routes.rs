use super::auth::CreatorToken;
use crate::db::Creator;
use crate::db::Imagefork;
use crate::db::Poster;
use crate::image_meta::ImageMetadata;
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::{http::Status, State};
use rocket_db_pools::{sqlx, Connection};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_creator,
        post_creator,
        get_posters,
        get_poster,
        post_poster
    ]
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Sql: {0}")]
    Sql(#[from] sqlx::Error),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        (Status::InternalServerError, self).respond_to(req)
    }
}

#[get("/creator")]
async fn get_creator(
    mut db: Connection<Imagefork>,
    token: &CreatorToken,
) -> Result<Option<Json<Creator>>, Error> {
    Ok(Creator::get(&mut db, token.id).await?.map(Into::into))
}

#[post("/creator")]
async fn post_creator() {}

#[get("/poster")]
async fn get_posters(
    token: &CreatorToken,
    mut db: Connection<Imagefork>,
) -> Result<Json<Vec<Poster>>, Error> {
    Ok(Poster::get_all_by_creator(&mut db, token.id).await?.into())
}

#[get("/poster/<id>")]
async fn get_poster(
    token: &CreatorToken,
    mut db: Connection<Imagefork>,
    id: i64,
) -> Result<Option<Json<Poster>>, Error> {
    Ok(Poster::get(&mut db, id, token.id).await?.map(Into::into))
}

#[derive(Serialize, Deserialize)]
struct PosterCreate {
    url: String,
}

#[post("/poster", format = "json", data = "<poster>")]
async fn post_poster(
    token: &CreatorToken,
    c: &State<ImageMetadata>,
    mut db: Connection<Imagefork>,
    poster: Json<PosterCreate>,
) -> Result<Json<Poster>, Error> {
    if token.lockout {
        todo!()
    }

    if let Some(false) = Creator::can_add_posters(&mut db, token.id).await? {
        todo!();
    }

    let metadata = c.get_metadata(&poster.url).await.unwrap();
    if let Some(poster) = Poster::post(&mut db, token.id, &poster.url, &metadata).await? {
        Ok(poster.into())
    } else {
        todo!()
    }
}
