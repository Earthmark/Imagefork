use super::Imagefork;
use crate::schema::creators::dsl::*;
use chrono::NaiveDateTime;
use rocket_db_pools::{diesel::prelude::*, Connection};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::creators)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Creator {
    pub id: i64,
    pub email: String,
    pub creation_time: NaiveDateTime,
    pub lockout: bool,
    pub moderator: bool,
    pub poster_limit: i32,
}

impl Creator {
    pub async fn get(
        db: &mut Connection<Imagefork>,
        creator_id: i64,
    ) -> crate::error::Result<Option<Self>> {
        Ok(creators
            .find(creator_id)
            .select(Creator::as_select())
            .first(db)
            .await
            .optional()?)
    }
}

#[cfg(test)]
pub mod test {
    use super::{
        super::{creator_token::test::*, Imagefork},
        Creator,
    };
    use crate::db::CreatorToken;
    use crate::test::TestRocket;
    use rocket::{serde::json::Json, Route};
    use rocket_db_pools::Connection;

    pub fn routes() -> Vec<Route> {
        routes![get_creator]
    }

    #[get("/test/get-creator?<id>")]
    pub async fn get_creator(mut db: Connection<Imagefork>, id: i64) -> Option<Json<Creator>> {
        Creator::get(&mut db, id).await.unwrap().map(Into::into)
    }

    #[test]
    fn new_user_has_defaults() {
        let client = TestRocket::default().client();
        client.get(uri!(delete_creator(email_addr = "c1")));
        let token: CreatorToken = client.get_json(uri!(login(email = "c1")));

        let creator: Option<Creator> = client.get_maybe_json(uri!(get_creator(id = token.id)));

        assert!(creator.is_some());
        let creator = creator.unwrap();
        assert_eq!(creator.moderator, false);
        assert_eq!(creator.lockout, false);
        assert_eq!(creator.email, "c1");
        assert!(creator.poster_limit < 20);
    }
}
