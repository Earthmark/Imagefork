use super::DbPool;
use sha2::{Digest, Sha256};
use tracing::instrument;

#[derive(Clone, Debug)]
pub struct Creator {
    pub id: i64,
    pub email_hash: Vec<u8>,
}

impl Creator {
    #[instrument(skip(db))]
    pub async fn get_by_email(db: &DbPool, email: &str) -> crate::error::Result<Option<Self>> {
        Ok(sqlx::query_as!(
            Self,
            r#"
        SELECT id, email_hash
        FROM creators
        WHERE email = $1
        "#,
            email
        )
        .fetch_optional(db)
        .await?)
    }

    #[instrument(skip(db))]
    pub async fn create_by_email(db: &DbPool, email: &str) -> crate::error::Result<Self> {
        let mut hasher = Sha256::new();
        hasher.update(email);
        // TODO: Possibly make this random every time, because it's in the DB, we just need to generate this safely.
        const SALT: &str = "Technically this is a valid salt... but it's quite long. w4htr5g[9o0jin12ukm2h,q3e4t5rwg2a-908[ihy2jure3gtj1h0[ds2vc2bj0mp3io[]";
        hasher.update(SALT);
        let hash = hasher.finalize().to_vec();

        Ok(sqlx::query_as!(
            Self,
            r#"
        INSERT INTO creators (email, email_hash)
        VALUES ($1, $2)
        RETURNING id, email_hash
        "#,
            email,
            hash
        )
        .fetch_one(db)
        .await?)
    }
}

/*
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
*/
