use super::DbPool;
use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;
use tracing::instrument;

#[derive(Deserialize, Serialize, Debug)]
pub struct Poster {
    id: i64,
    creator: i64,
    creation_time: PrimitiveDateTime,
    stopped: bool,
    lockout: bool,
    servable: bool,
}

impl Poster {
    pub async fn _get(
        db: &DbPool,
        creator_id: i64,
        poster_id: i64,
    ) -> crate::error::Result<Option<Self>> {
        Ok(sqlx::query_as!(
            Poster,
            r#"
        SELECT *
        FROM posters
        WHERE id = $1 AND creator = $2
        "#,
            poster_id,
            creator_id
        )
        .fetch_optional(db)
        .await?)
    }

    pub async fn _get_all_by_creator(
        db: &DbPool,
        creator_id: i64,
    ) -> crate::error::Result<Vec<Self>> {
        Ok(sqlx::query_as!(
            Poster,
            r#"
        SELECT *
        FROM posters
        WHERE creator = $1
        "#,
            creator_id
        )
        .fetch_all(db)
        .await?)
    }

    pub async fn _create(db: &DbPool, creator_id: i64) -> crate::error::Result<Option<Self>> {
        Ok(sqlx::query_as!(
            Poster,
            r#"
        INSERT INTO posters (creator)
        VALUES ($1)
        RETURNING *
        "#,
            creator_id
        )
        .fetch_optional(db)
        .await?)
    }

    pub async fn _update(
        db: &DbPool,
        creator_id: i64,
        poster_id: i64,
        is_stopped: bool,
    ) -> crate::error::Result<Option<Self>> {
        Ok(sqlx::query_as!(
            Self,
            r#"
        UPDATE posters
        SET stopped = $3
        WHERE id = $1 AND creator = $2
        RETURNING *
        "#,
            creator_id,
            poster_id,
            is_stopped
        )
        .fetch_optional(db)
        .await?)
    }

    pub async fn _delete(
        db: &DbPool,
        creator_id: i64,
        poster_id: i64,
    ) -> crate::Result<Option<Self>> {
        Ok(sqlx::query_as!(
            Self,
            r#"
        DELETE FROM posters
        WHERE id = $1 AND creator = $2
        RETURNING *
        "#,
            creator_id,
            poster_id
        )
        .fetch_optional(db)
        .await?)
    }

    #[instrument(skip(db))]
    pub async fn get_id_of_approx(db: &DbPool) -> crate::Result<Option<i64>> {
        Ok(sqlx::query!(
            r#"
            SELECT id
            FROM posters
            WHERE servable
            ORDER BY random()
            LIMIT 1
        "#
        )
        .fetch_optional(db)
        .await?
        .map(|r| r.id))
    }
}

/*
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
*/
