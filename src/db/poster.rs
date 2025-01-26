use super::DbConn;
use crate::schema::posters::dsl;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;
use tracing::instrument;

#[derive(Queryable, Selectable, Deserialize, Serialize, Debug)]
#[diesel(table_name = crate::schema::posters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Poster {
    id: i64,
    creator: i64,
    creation_time: PrimitiveDateTime,
    stopped: bool,
    lockout: bool,
    servable: bool,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::posters)]
struct NewPoster {
    creator: i64,
}

define_sql_function!(fn random() -> Text);

impl Poster {
    pub async fn _get(
        db: &mut DbConn,
        creator_id: i64,
        poster_id: i64,
    ) -> crate::error::Result<Option<Self>> {
        Ok(dsl::posters
            .filter(dsl::id.eq(poster_id).and(dsl::creator.eq(creator_id)))
            .select(Self::as_select())
            .first(&mut *db)
            .await
            .optional()?)
    }

    pub async fn _get_all_by_creator(
        db: &mut DbConn,
        creator_id: i64,
    ) -> crate::error::Result<Vec<Self>> {
        Ok(dsl::posters
            .filter(dsl::creator.eq(creator_id))
            .order_by(dsl::id.desc())
            .select(Poster::as_select())
            .load(db)
            .await?)
    }

    pub async fn _create(db: &mut DbConn, creator_id: i64) -> crate::error::Result<Option<Self>> {
        Ok(diesel::insert_into(dsl::posters)
            .values(NewPoster {
                creator: creator_id,
            })
            .returning(Self::as_returning())
            .get_result(db)
            .await
            .optional()?)
    }

    pub async fn _update(
        db: &mut DbConn,
        creator_id: i64,
        poster_id: i64,
        is_stopped: bool,
    ) -> crate::error::Result<Option<Self>> {
        Ok(diesel::update(
            dsl::posters
                .find(poster_id)
                .filter(dsl::creator.eq(creator_id)),
        )
        .set(dsl::stopped.eq(is_stopped))
        .returning(Self::as_returning())
        .get_result(db)
        .await
        .optional()?)
    }

    pub async fn _delete(
        db: &mut DbConn,
        creator_id: i64,
        poster_id: i64,
    ) -> crate::Result<Option<Self>> {
        Ok(diesel::delete(
            dsl::posters
                .find(poster_id)
                .filter(dsl::creator.eq(creator_id)),
        )
        .returning(Self::as_returning())
        .get_result(db)
        .await
        .optional()?)
    }

    #[instrument(skip(db))]
    pub async fn get_id_of_approx(db: &mut DbConn) -> crate::Result<Option<i64>> {
        Ok(dsl::posters
            .select(dsl::id)
            .filter(dsl::servable)
            .order_by(random())
            .first(db)
            .await
            .optional()?)
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
