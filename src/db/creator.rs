use super::Imagefork;
use chrono::NaiveDateTime;
use rocket_db_pools::{sqlx, Connection};
use serde::{Deserialize, Serialize};
use sqlx::Result;

#[derive(Deserialize, Serialize, Debug)]
pub struct Creator {
    id: i64,
    email: String,
    creation_time: NaiveDateTime,
    lockout: bool,
    moderator: bool,
    pub poster_limit: i32,
}

impl Creator {
    pub async fn get(db: &mut Connection<Imagefork>, id: i64) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, email, creation_time, lockout, moderator, poster_limit
            FROM Creators WHERE id = $1 LIMIT 1",
            id
        )
        .fetch_optional(&mut **db)
        .await
    }

    pub async fn can_add_posters(
        db: &mut Connection<Imagefork>,
        creator_id: i64,
    ) -> Result<Option<bool>> {
        struct CanAddPoster {
            can_add: bool,
        }

        Ok(sqlx::query_as!(
            CanAddPoster,
            r#"SELECT poster_limit > (SELECT COUNT(*) FROM Posters WHERE creator = Creators.id) AS "can_add!"
            FROM Creators WHERE id = $1
            "#,
            creator_id,
        )
        .fetch_optional(&mut **db)
        .await?
        .map(|c| c.can_add))
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
    use rocket::serde::json::Json;
    use rocket_db_pools::Connection;

    #[get("/test/get-creator?<id>")]
    pub async fn get_creator(mut db: Connection<Imagefork>, id: i64) -> Option<Json<Creator>> {
        Creator::get(&mut db, id).await.unwrap().map(Into::into)
    }

    #[test]
    fn new_user_has_defaults() {
        let client = TestRocket::new(routes![delete_creator, login, get_creator]).client();
        client.get(uri!(delete_creator(email = "c1")));
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
