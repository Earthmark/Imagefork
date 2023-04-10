use crate::db::Creator;
use crate::db::CreatorToken;
use crate::db::Imagefork;
use crate::db::Poster;
use crate::{Error, Error::*, Result};
use rocket::form::Form;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket_db_pools::Connection;
use serde::Deserialize;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_posters,
        get_poster,
        post_poster_json,
        post_poster_html,
        put_poster_json,
        put_poster_html
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

async fn post_poster(
    token: &CreatorToken,
    mut db: Connection<Imagefork>,
    poster: &PosterCreate,
) -> Result<Poster> {
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
    Ok(poster)
}

#[post("/posters", format = "json", data = "<poster>")]
async fn post_poster_json(
    token: &CreatorToken,
    db: Connection<Imagefork>,
    poster: Json<PosterCreate>,
) -> Result<Json<Poster>> {
    Ok(post_poster(token, db, &poster).await?.into())
}

#[post("/posters", data = "<poster>", rank = 2)]
async fn post_poster_html(
    token: &CreatorToken,
    db: Connection<Imagefork>,
    poster: Form<PosterCreate>,
) -> Result<Redirect> {
    post_poster(token, db, &poster).await?;
    Ok(Redirect::to(uri!(super::ui::posters)))
}

#[derive(Deserialize, FromForm)]
struct PosterModify {
    stopped: bool,
}

async fn put_poster(
    token: &CreatorToken,
    mut db: Connection<Imagefork>,
    id: i64,
    poster: &PosterModify,
) -> Result<Poster> {
    if token.lockout {
        return Err(LockedOut);
    }

    let poster = Poster::update(&mut db, token.id, id, poster.stopped)
        .await?
        .ok_or(Error::internal_from("Failed to create poster."))?;
    Ok(poster)
}

#[put("/posters/<id>", data = "<poster>")]
async fn put_poster_json(
    token: &CreatorToken,
    db: Connection<Imagefork>,
    id: i64,
    poster: Json<PosterModify>,
) -> Result<Json<Poster>> {
    Ok(put_poster(token, db, id, &poster).await?.into())
}

#[post("/posters/<id>", data = "<poster>", rank = 2)]
async fn put_poster_html(
    token: &CreatorToken,
    db: Connection<Imagefork>,
    id: i64,
    poster: Form<PosterModify>,
) -> Result<Redirect> {
    put_poster(token, db, id, &poster).await?;
    Ok(Redirect::to(uri!(super::ui::posters)))
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
