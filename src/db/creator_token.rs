use super::Imagefork;
use base64::Engine;
use chrono::{DateTime, NaiveDateTime, Utc};
use rand::RngCore;
use rocket_db_pools::{sqlx, Connection};
use serde::{Deserialize, Serialize};
use sqlx::Result;

#[derive(Deserialize, Serialize)]
pub struct CreatorToken {
    pub id: i64,
    pub token: String,
    minting_time: NaiveDateTime,
    pub moderator: bool,
    pub lockout: bool,
}

fn generate_token() -> String {
    let mut token = [0; 32];
    rand::thread_rng().try_fill_bytes(&mut token).unwrap();
    base64::engine::general_purpose::URL_SAFE.encode(token)
}

impl CreatorToken {
    pub fn minting_time(&self) -> DateTime<Utc> {
        DateTime::from_utc(self.minting_time, Utc)
    }

    pub async fn get_by_token(db: &mut Connection<Imagefork>, token: &str) -> Result<Option<Self>> {
        let token = sqlx::query_as!(
            Self,
            r#"SELECT id, token AS "token!", minting_time AS "minting_time!", moderator, lockout
          FROM Creators WHERE token = $1 LIMIT 1"#,
            token
        )
        .fetch_optional(&mut **db)
        .await?;

        Ok(token)
    }

    pub async fn relogin(db: &mut Connection<Imagefork>, id: i64) -> Result<Option<Self>> {
        let token = generate_token();

        sqlx::query_as!(
            Self,
            r#"UPDATE Creators
            SET token = $1, minting_time = (now() at time zone 'utc')
            WHERE id = $2
            RETURNING id, token AS "token!", minting_time AS "minting_time!", moderator, lockout"#,
            token,
            id
        )
        .fetch_optional(&mut **db)
        .await
    }

    pub async fn login(db: &mut Connection<Imagefork>, email: &str) -> Result<Self> {
        let token = generate_token();

        sqlx::query_as!(
          Self,
          r#"INSERT INTO Creators (email, token, minting_time) VALUES ($1, $2, (now() at time zone 'utc'))
          ON CONFLICT (email)
          DO UPDATE SET token = $2, minting_time = (now() at time zone 'utc')
          RETURNING id, token AS "token!", minting_time AS "minting_time!", moderator, lockout"#,
          email,
          token,
      )
      .fetch_one(&mut **db)
      .await
    }
}

#[cfg(test)]
pub mod test {
    use std::time::Duration;

    use super::{super::Imagefork, CreatorToken};
    use crate::portal::auth::test::*;
    use crate::test::{TestClient, TestRocket};
    use rocket::{serde::json::Json, Route};
    use rocket_db_pools::{sqlx, Connection};

    pub fn routes() -> Vec<Route> {
        routes![delete_creator, login, promote]
    }

    #[get("/test/delete_creator?<email>")]
    pub async fn delete_creator(mut db: Connection<Imagefork>, email: String) {
        sqlx::query!(
            "DELETE FROM Creators 
            WHERE email = $1",
            email
        )
        .execute(&mut *db)
        .await
        .unwrap();
    }

    #[get("/test/promote?<id>")]
    pub async fn promote(mut db: Connection<Imagefork>, id: i64) {
        sqlx::query!(
            "UPDATE Creators 
            SET moderator = true
            WHERE id = $1",
            id
        )
        .execute(&mut *db)
        .await
        .unwrap();
    }

    #[get("/test/login?<email>")]
    pub async fn login(mut db: Connection<Imagefork>, email: String) -> Json<CreatorToken> {
        CreatorToken::login(&mut db, &email).await.unwrap().into()
    }

    #[get("/test/relogin?<id>")]
    async fn relogin(mut db: Connection<Imagefork>, id: i64) -> Option<Json<CreatorToken>> {
        CreatorToken::relogin(&mut db, id)
            .await
            .unwrap()
            .map(Into::into)
    }

    #[get("/test/get-by-token?<token>")]
    async fn get_by_token(
        mut db: Connection<Imagefork>,
        token: String,
    ) -> Option<Json<CreatorToken>> {
        CreatorToken::get_by_token(&mut db, &token)
            .await
            .unwrap()
            .map(Into::into)
    }

    impl TestClient {
        pub fn creator(&self, email: &'static str) -> TestUser {
            self.get(uri!(delete_creator(email = email)));
            let bearer = self.get_json(uri!(login(email = email)));
            TestUser {
                email,
                bearer,
                client: self,
            }
        }

        fn delete_creator(&self, email: &str) {
            self.get(uri!(delete_creator(email = email)));
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
            self.client.get(uri!(promote(id = self.id())));
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
