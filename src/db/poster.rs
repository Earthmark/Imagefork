use super::Imagefork;
use chrono::NaiveDateTime;
use rocket_db_pools::{sqlx, Connection};
use serde::{Deserialize, Serialize};
use sqlx::Result;

#[derive(Deserialize, Serialize, Debug)]
pub struct Poster {
    id: i64,
    creator: i64,
    creation_time: NaiveDateTime,
    url: String,
    stopped: bool,
    lockout: bool,
    serveable: bool,
}

impl Poster {
    pub async fn get(
        db: &mut Connection<Imagefork>,
        creator_id: i64,
        poster_id: i64,
    ) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM Posters WHERE id = $1 AND creator = $2 LIMIT 1",
            poster_id,
            creator_id
        )
        .fetch_optional(&mut **db)
        .await
    }

    pub async fn get_all_by_creator(
        db: &mut Connection<Imagefork>,
        creator_id: i64,
    ) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM Posters WHERE creator = $1 ORDER BY id",
            creator_id
        )
        .fetch_all(&mut **db)
        .await
    }

    pub async fn create(
        db: &mut Connection<Imagefork>,
        creator_id: i64,
        url: &str,
    ) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "INSERT INTO Posters (Creator, Url)
            SELECT $1, $2
            RETURNING *;
            ",
            creator_id,
            url,
        )
        .fetch_optional(&mut **db)
        .await
    }

    pub async fn update(
        db: &mut Connection<Imagefork>,
        creator_id: i64,
        poster_id: i64,
        stopped: bool,
    ) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "UPDATE Posters
            SET stopped = $3
            WHERE id = $1 AND creator = $2
            RETURNING *;
            ",
            poster_id,
            creator_id,
            stopped,
        )
        .fetch_optional(&mut **db)
        .await
    }

    pub async fn delete(
        db: &mut Connection<Imagefork>,
        creator_id: i64,
        poster_id: i64,
    ) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "DELETE FROM Posters
            WHERE id = $1 AND creator = $2
            RETURNING *;
            ",
            poster_id,
            creator_id,
        )
        .fetch_optional(&mut **db)
        .await
    }

    pub async fn get_id_of_approx(db: &mut Connection<Imagefork>) -> Result<Option<i64>> {
        struct FoundPoster {
            id: i64,
        }
        Ok(sqlx::query_as!(
            FoundPoster,
            "SELECT id FROM Posters WHERE serveable ORDER BY RANDOM() LIMIT 1"
        )
        .fetch_optional(&mut **db)
        .await?
        .map(|f| f.id))
    }

    pub async fn get_url(db: &mut Connection<Imagefork>, id: i64) -> Result<Option<String>> {
        struct PosterUrl {
            url: String,
        }
        Ok(
            sqlx::query_as!(PosterUrl, "SELECT url FROM Posters WHERE id = $1", id)
                .fetch_optional(&mut **db)
                .await?
                .map(|f| f.url),
        )
    }
}

#[cfg(test)]
mod test {
    use super::{super::Imagefork, Poster};
    use crate::test::TestRocket;
    use rocket::serde::json::Json;
    use rocket_db_pools::Connection;

    #[get("/test/get-poster?<poster_id>&<creator_id>")]
    async fn get_poster(
        mut db: Connection<Imagefork>,
        poster_id: i64,
        creator_id: i64,
    ) -> Option<Json<Poster>> {
        Poster::get(&mut db, creator_id, poster_id)
            .await
            .unwrap()
            .map(Into::into)
    }

    #[get("/test/get-all?<creator_id>")]
    async fn get_all_for(mut db: Connection<Imagefork>, creator_id: i64) -> Json<Vec<Poster>> {
        Poster::get_all_by_creator(&mut db, creator_id)
            .await
            .unwrap()
            .into()
    }

    #[get("/test/add_poster?<creator_id>&<url>")]
    async fn add_poster(
        mut db: Connection<Imagefork>,
        creator_id: i64,
        url: &str,
    ) -> Option<Json<Poster>> {
        Poster::create(&mut db, creator_id, url)
            .await
            .unwrap()
            .map(Into::into)
    }

    #[test]
    fn new_user_has_no_posters() {
        let client = TestRocket::new(routes![get_poster, get_all_for, add_poster]).client();
        let token = client.creator("p1");

        let posters: Vec<Poster> = client.get_json(uri!(get_all_for(creator_id = token.id())));
        assert!(posters.len() == 0);
    }
}
