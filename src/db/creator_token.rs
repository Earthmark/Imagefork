use super::DbConn;
use base64::Engine;
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::dsl::now;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::creators)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreatorToken {
    pub id: i64,
    pub moderator: bool,
    pub lockout: bool,
}

fn generate_token() -> String {
    let mut token = [0; 32];
    rand::thread_rng().try_fill_bytes(&mut token).unwrap();
    base64::engine::general_purpose::URL_SAFE.encode(token)
}

impl CreatorToken {
}

/*
#[cfg(test)]
pub mod test {
    use std::time::Duration;

    use super::{super::Imagefork, CreatorToken};
    use crate::portal::auth::test::*;
    use crate::test::{TestClient, TestRocket};
    use ::diesel::query_dsl::methods::{FilterDsl, FindDsl};
    use ::diesel::ExpressionMethods;
    use diesel_async::RunQueryDsl;
    use rocket::{serde::json::Json, Route};
    use rocket_db_pools::{diesel, Connection};

    pub fn routes() -> Vec<Route> {
        routes![delete_creator, login, promote]
    }

    #[get("/test/delete_creator?<email_addr>")]
    pub async fn delete_creator(mut db: DbConn, email_addr: String) {
        use crate::schema::creators::dsl::*;
        diesel::delete(creators.filter(email.eq(email_addr)))
            .execute(&mut db)
            .await
            .unwrap();
    }

    #[get("/test/promote?<creator_id>")]
    pub async fn promote(mut db: DbConn, creator_id: i64) {
        use crate::schema::creators::dsl::*;
        diesel::update(creators.find(creator_id))
            .set(moderator.eq(true))
            .execute(&mut db)
            .await
            .unwrap();
    }

    #[get("/test/login?<email>")]
    pub async fn login(mut db: DbConn, email: String) -> Json<CreatorToken> {
        CreatorToken::login(&mut db, &email).await.unwrap().into()
    }

    #[get("/test/relogin?<id>")]
    async fn relogin(mut db: DbConn, id: i64) -> Option<Json<CreatorToken>> {
        CreatorToken::relogin(&mut db, id)
            .await
            .unwrap()
            .map(Into::into)
    }

    #[get("/test/get-by-token?<token>")]
    async fn get_by_token(
        mut db: DbConn,
        token: String,
    ) -> Option<Json<CreatorToken>> {
        CreatorToken::get_by_token(&mut db, &token)
            .await
            .unwrap()
            .map(Into::into)
    }

    impl TestClient {
        pub fn creator(&self, email: &'static str) -> TestUser {
            self.get(uri!(delete_creator(email_addr = email)));
            let bearer = self.get_json(uri!(login(email = email)));
            TestUser {
                email,
                bearer,
                client: self,
            }
        }

        fn delete_creator(&self, email: &str) {
            self.get(uri!(delete_creator(email_addr = email)));
        }
    }

    pub struct TestUser<'a> {
        email: &'static str,
        bearer: CreatorToken,
        client: &'a TestClient,
    }

    impl TestUser<'_> {
        pub fn id(&self) -> i64 {
            self.bearer.id
        }
        pub fn email(&self) -> &str {
            self.email
        }
        pub fn token(&self) -> &str {
            &self.bearer.token
        }

        pub fn login(&self) {
            self.client.get(uri!(force_login(id = self.id())));
        }

        pub fn promote(&self) {
            self.client.get(uri!(promote(creator_id = self.id())));
        }

        pub fn delete(&self) {
            self.client.delete_creator(self.email());
        }
    }

    impl Drop for TestUser<'_> {
        fn drop(&mut self) {
            self.delete();
        }
    }

    #[test]
    fn login_creates_user() {
        let client = TestRocket::new(routes![relogin, get_by_token]).client();
        let token = client.creator("ct1");
        assert!(token.token().len() > 10);

        let gotten_token: Option<CreatorToken> =
            client.get_maybe_json(uri!(get_by_token(token = &token.token())));
        assert!(gotten_token.is_some());

        token.delete();
        let gotten_token: Option<CreatorToken> =
            client.get_maybe_json(uri!(get_by_token(token = &token.token())));

        assert!(gotten_token.is_none());
    }

    #[test]
    fn relog_resets_minting_time() {
        let client = TestRocket::new(routes![relogin, get_by_token]).client();
        let token = client.creator("ct2");
        let old_time = token.bearer.minting_time;

        std::thread::sleep(Duration::from_millis(500));

        let token: CreatorToken = client.get_json(uri!(relogin(id = token.id())));

        assert!(old_time < token.minting_time);
    }

    #[test]
    fn relog_unknown_token() {
        let client = TestRocket::new(routes![relogin, get_by_token]).client();

        let gotten_token: Option<CreatorToken> =
            client.get_maybe_json(uri!(get_by_token(token = "A very unknown token")));

        assert!(gotten_token.is_none());
    }
}
*/
