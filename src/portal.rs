use chrono::NaiveDateTime;
use rocket::outcome::{try_outcome, IntoOutcome};
use rocket::request::{FromRequest, Outcome};
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::{fairing, Build, Request, Rocket};
use rocket::{http::Status, State};
use rocket_db_pools::{sqlx, Connection, Database};
use serde::{Deserialize, Serialize};
use sqlx::migrate;
use thiserror::Error;

use crate::image_meta::{ImageMetadata, Metadata};

#[derive(Database)]
#[database("imagefork")]
pub struct Imagefork(pub sqlx::SqlitePool);

impl Imagefork {
    pub async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
        if let Some(db) = Self::fetch(&rocket) {
            if let Err(e) = migrate!().run(&db.0).await {
                warn!("Failed to migrate DB: {}", e);
                Err(rocket)
            } else {
                Ok(rocket)
            }
        } else {
            Err(rocket)
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Creator {
    id: i64,
    email: String,
    creation_time: NaiveDateTime,
    referal_token: Option<String>,
    lockout: bool,
    moderator: bool,
    poster_limit: i64,
}

impl Creator {
    async fn get(db: &mut Connection<Imagefork>, id: i64) -> Result<Option<Self>> {
        Ok(
            sqlx::query_as!(Self, "SELECT * FROM Creators WHERE id = ?", id)
                .fetch_optional(&mut **db)
                .await?,
        )
    }
}

#[derive(Clone)]
pub struct CreatorToken {
    id: i64,
    lockout: bool,
    moderator: bool,
}

impl CreatorToken {
    async fn get(db: &mut Connection<Imagefork>, id: i64) -> Result<Option<Self>> {
        Ok(sqlx::query_as!(
            Self,
            "SELECT id, lockout, moderator FROM Creators WHERE id = ?",
            id
        )
        .fetch_optional(&mut **db)
        .await?)
    }

    fn is_moderator(&self) -> bool {
        self.moderator
    }
}

#[async_trait]
impl<'r> FromRequest<'r> for &'r CreatorToken {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user_ref = request
            .local_cache_async(async {
                let mut db = request.guard::<Connection<Imagefork>>().await.succeeded()?;
                if let Some(id) = request
                    .cookies()
                    .get_private("creator_id")
                    .and_then(|cookie| cookie.value().parse().ok())
                {
                    CreatorToken::get(&mut db, id).await.ok().flatten()
                } else {
                    None
                }
            })
            .await;
        user_ref.as_ref().or_forward(())
    }
}

struct ModeratorToken<'a>(&'a CreatorToken);

#[async_trait]
impl<'r> FromRequest<'r> for ModeratorToken<'r> {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let creator = try_outcome!(request.guard::<&CreatorToken>().await);
        if creator.is_moderator() {
            Outcome::Success(ModeratorToken(&creator))
        } else {
            Outcome::Forward(())
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Poster {
    id: i64,
    creator: i64,
    creation_time: NaiveDateTime,
    url: String,
    height: i64,
    width: i64,
    hash: String,
    dead_url: bool,
    life_last_checked: NaiveDateTime,
    start_time: NaiveDateTime,
    end_time: Option<NaiveDateTime>,
    stopped: bool,
    lockout: bool,
}

struct PosterCount {
    poster_limit: i64,
    posters: i64,
}

impl Poster {
    async fn get(
        db: &mut Connection<Imagefork>,
        poster_id: i64,
        creator_id: i64,
    ) -> Result<Option<Self>> {
        Ok(sqlx::query_as!(
            Self,
            "SELECT * FROM Posters WHERE id = ? AND creator =  ?",
            poster_id,
            creator_id
        )
        .fetch_optional(&mut **db)
        .await?)
    }

    async fn poster_counts(
        db: &mut Connection<Imagefork>,
        creator_id: i64,
    ) -> Result<Option<PosterCount>> {
        Ok(sqlx::query_as!(
            PosterCount,
            "SELECT poster_limit, (SELECT COUNT(*) FROM Posters WHERE creator = id) AS posters
            FROM Creators WHERE id = ?
            LIMIT 1;
            ",
            creator_id,
        )
        .fetch_optional(&mut **db)
        .await?)
    }

    async fn post(
        db: &mut Connection<Imagefork>,
        creator_id: i64,
        url: &str,
        metadata: &Metadata,
    ) -> Result<Option<Self>> {
        Ok(sqlx::query_as!(
            Self,
            "INSERT INTO Posters (Creator, Url, Height, Width, Hash)
            SELECT ?, ?, ?, ?, ?
            WHERE (SELECT COUNT(*) FROM Posters WHERE creator = ?) < (SELECT poster_limit FROM Creators WHERE id = ? LIMIT 1)
            RETURNING *;
            ",
            creator_id,
            url,
            metadata.height,
            metadata.width,
            metadata.hash,
            creator_id,
            creator_id,
        )
        .fetch_optional(&mut **db)
        .await?)
    }
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

type Result<T> = std::result::Result<T, Error>;

#[get("/creator/<id>")]
pub async fn get_creator(mut db: Connection<Imagefork>, id: i64) -> Result<Option<Json<Creator>>> {
    Ok(Creator::get(&mut db, id).await?.map(Into::into))
}

#[post("/creator")]
pub async fn new_creator() {}

#[derive(Serialize, Deserialize)]
pub struct PosterCreate {
    url: String,
}

#[get("/poster/<id>")]
pub async fn get_poster(
    token: &CreatorToken,
    mut db: Connection<Imagefork>,
    id: i64,
) -> Result<Option<Json<Poster>>> {
    Ok(Poster::get(&mut db, id, token.id).await?.map(Into::into))
}

#[post("/poster", format = "json", data = "<poster>")]
pub async fn new_poster(
    token: &CreatorToken,
    c: &State<ImageMetadata>,
    mut db: Connection<Imagefork>,
    poster: Json<PosterCreate>,
) -> Result<Json<Poster>> {
    if token.lockout {
        todo!()
    }
    if let Some(counts) = Poster::poster_counts(&mut db, token.id).await? {
        if counts.poster_limit > counts.posters {
            todo!();
        }
        todo!();
    }

    let metadata = c.get_metadata(&poster.url).await.unwrap();
    if let Some(poster) = Poster::post(&mut db, token.id, &poster.url, &metadata).await? {
        Ok(poster.into())
    } else {
        todo!()
    }
}
