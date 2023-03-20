use crate::db::Creator;
use crate::db::CreatorToken;
use crate::db::Imagefork;
use crate::db::Poster;
use crate::image_meta::WebImageMetadataAggregator;
use rocket::http::Status;
use rocket::response::status::Unauthorized;
use rocket::serde::json::Json;
use rocket::Either;
use rocket::State;
use rocket_db_pools::Connection;
use serde::Deserialize;

use crate::Result;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_posters,
        get_posters_no_token,
        get_poster,
        get_poster_no_token,
        post_poster,
        post_poster_no_token
    ]
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
    c: &State<WebImageMetadataAggregator>,
    mut db: Connection<Imagefork>,
    poster: Json<PosterCreate>,
) -> Result<Either<Json<Poster>, (Status, ())>> {
    if token.lockout {
        return Ok(Either::Right((Status::Unauthorized, ())));
    }

    if Creator::can_add_posters(&mut db, token.id).await? != Some(true) {
        return Ok(Either::Right((Status::Forbidden, ())));
    }

    let metadata = c.get_metadata(&poster.url).await.unwrap();
    let poster = Poster::post(&mut db, token.id, &poster.url, &metadata)
        .await?
        .ok_or(crate::Error::SystemError(
            "Failed to create poster.".to_string(),
        ))?;
    Ok(Either::Left(poster.into()))
}

#[post("/poster", format = "json", rank = 2)]
fn post_poster_no_token() -> Unauthorized<()> {
    Unauthorized(None)
}

#[cfg(test)]
mod test {
    use crate::{db::Poster, test::*};
    use rocket::http::StatusClass;

    #[test]
    fn creator_only_request_rejected() {
        let client = TestRocket::default().client();
        assert_eq!(
            client.get(uri!(super::get_posters)).class(),
            StatusClass::ClientError
        );
    }

    #[test]
    fn no_creator_logged_in() {
        let client = TestRocket::default().client();
        let user = client.creator("pc1");
        user.login();
        let creator: Vec<Poster> = client.get_json(uri!(super::get_posters()));
        assert_eq!(creator.len(), 0);
    }

    #[test]
    fn provides_logged_in_posters() {
        let client = TestRocket::default().client();
        let user = client.creator("pc2");
        user.login();
        let creator: Vec<Poster> = client.get_json(uri!(super::get_posters()));
        assert_eq!(creator.len(), 0);
        
        assert_eq!(
            client.get(uri!(super::get_posters)).class(),
            StatusClass::Success
        );
    }
}
