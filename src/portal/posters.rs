use crate::db::Creator;
use crate::db::CreatorToken;
use crate::db::Imagefork;
use crate::db::Poster;
use crate::{Error, Error::*, Result};
use rocket::serde::json::Json;
use rocket_db_pools::Connection;
use serde::Deserialize;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_posters,
        get_poster,
        post_poster,
        put_poster,
        delete_poster,
    ]
}

#[get("/posters", format = "json")]
async fn get_posters(
    token: &CreatorToken,
    mut db: Connection<Imagefork>,
) -> Result<Json<Vec<Poster>>> {
    Ok(Poster::get_all_by_creator(&mut db, token.id).await?.into())
}

#[get("/posters/<id>", format = "json")]
async fn get_poster(
    token: &CreatorToken,
    mut db: Connection<Imagefork>,
    id: i64,
) -> Result<Option<Json<Poster>>> {
    Ok(Poster::get(&mut db, id, token.id).await?.map(Into::into))
}

#[derive(Deserialize, FromForm)]
struct PosterCreate {
    url: String,
}

#[post("/posters", format = "json", data = "<poster>")]
async fn post_poster(
    token: &CreatorToken,
    mut db: Connection<Imagefork>,
    poster: Json<PosterCreate>,
) -> Result<Json<Poster>> {
    if token.lockout {
        return Err(LockedOut);
    }

    if Creator::can_add_posters(&mut db, token.id).await? != Some(true) {
        return Err(TooManyPosters);
    }

    // TODO: Verify a poster can be retrieved.

    let poster = Poster::create(&mut db, token.id, &poster.url)
        .await?
        .ok_or(Error::internal_from("Failed to create poster."))?;
    Ok(poster.into())
}

#[derive(Deserialize, FromForm)]
struct PosterModify {
    stopped: bool,
}

#[put("/posters/<id>", data = "<poster>")]
async fn put_poster(
    token: &CreatorToken,
    mut db: Connection<Imagefork>,
    id: i64,
    poster: Json<PosterModify>,
) -> Result<Json<Poster>> {
    if token.lockout {
        return Err(LockedOut);
    }

    let poster = Poster::update(&mut db, token.id, id, poster.stopped)
        .await?
        .ok_or(Error::internal_from("Failed update poster."))?;
    Ok(poster.into())
}

#[delete("/posters/<id>")]
async fn delete_poster(
    token: &CreatorToken,
    mut db: Connection<Imagefork>,
    id: i64,
) -> Result<Json<Poster>> {
    if token.lockout {
        return Err(LockedOut);
    }

    let poster = Poster::delete(&mut db, token.id, id)
        .await?
        .ok_or(Error::internal_from("Failed update poster."))?;
    Ok(poster.into())
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
