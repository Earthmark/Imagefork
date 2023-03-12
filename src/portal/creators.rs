use crate::db::CreatorToken;
use crate::db::Creator;
use crate::db::Imagefork;
use crate::db::Poster;
use crate::image_meta::ImageMetadata;
use rocket::response::status::Forbidden;
use rocket::response::status::Unauthorized;
use rocket::serde::json::Json;
use rocket::Either;
use rocket::State;
use rocket_db_pools::Connection;
use serde::Deserialize;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_creator,
        get_creator_no_token,
        get_posters,
        get_posters_no_token,
        get_poster,
        get_poster_no_token,
        post_poster,
        post_poster_no_token
    ]
}

use crate::Result;

#[get("/creator", format = "json")]
async fn get_creator(
    mut db: Connection<Imagefork>,
    token: &CreatorToken,
) -> Result<Option<Json<Creator>>> {
    Ok(Creator::get(&mut db, token.id).await?.map(Into::into))
}

#[get("/creator", format = "json", rank = 2)]
fn get_creator_no_token() -> Unauthorized<()> {
    Unauthorized(None)
}

#[get("/poster", format = "json")]
async fn get_posters(
    token: &CreatorToken,
    mut db: Connection<Imagefork>,
) -> Result<Json<Vec<Poster>>> {
    Ok(Poster::get_all_by_creator(&mut db, token.id).await?.into())
}

#[get("/poster", format = "json", rank = 2)]
fn get_posters_no_token() -> Unauthorized<()> {
    Unauthorized(None)
}

#[get("/poster/<id>", format = "json")]
async fn get_poster(
    token: &CreatorToken,
    mut db: Connection<Imagefork>,
    id: i64,
) -> Result<Option<Json<Poster>>> {
    Ok(Poster::get(&mut db, id, token.id).await?.map(Into::into))
}

#[get("/poster/<_>", format = "json", rank = 2)]
fn get_poster_no_token() -> Unauthorized<()> {
    Unauthorized(None)
}

#[derive(Deserialize)]
struct PosterCreate {
    url: String,
}

#[post("/poster", format = "json", data = "<poster>")]
async fn post_poster(
    token: &CreatorToken,
    c: &State<ImageMetadata>,
    mut db: Connection<Imagefork>,
    poster: Json<PosterCreate>,
) -> Result<Either<Json<Poster>, Either<Unauthorized<()>, Forbidden<()>>>> {
    if token.lockout {
        return Ok(Either::Right(Either::Left(Unauthorized(None))));
    }

    if Creator::can_add_posters(&mut db, token.id).await? != Some(true) {
        return Ok(Either::Right(Either::Right(Forbidden(None))));
    }

    let metadata = c.get_metadata(&poster.url).await.unwrap();
    if let Some(poster) = Poster::post(&mut db, token.id, &poster.url, &metadata).await? {
        Ok(Either::Left(poster.into()))
    } else {
        todo!()
    }
}

#[post("/poster", format = "json", rank = 2)]
fn post_poster_no_token() -> Unauthorized<()> {
    Unauthorized(None)
}
