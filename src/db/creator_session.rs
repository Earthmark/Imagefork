use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

use super::{util::generate_token, DbConn};
use crate::schema::creator_sessions::dsl::creator_sessions;

#[derive(Queryable, Selectable, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::creator_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreatorSession {
    pub creator: i64,
    pub token: String,
    pub creation_time: NaiveDateTime,
}

#[derive(Insertable, Queryable)]
#[diesel(table_name = crate::schema::creator_sessions)]
struct SessionCookie {
    creator: i64,
    token: String,
}

impl From<CreatorSession> for SessionCookie {
    fn from(value: CreatorSession) -> Self {
        Self {
            creator: value.creator,
            token: value.token,
        }
    }
}

fn split_cookie(cookie: &str) -> Option<[&str; 2]> {
    let mut split_cookie = cookie.split("+");
    let result = [split_cookie.next()?, split_cookie.next()?];
    if let None = split_cookie.next() {
        Some(result)
    } else {
        None
    }
}

impl SessionCookie {
    fn new(creator: i64) -> Self {
        Self {
            creator,
            token: generate_token(),
        }
    }

    fn to_cookie(&self) -> String {
        format!("{}+{}", &self.creator, self.token)
    }

    fn from_cookie(cookie: &str) -> crate::Result<Self> {
        let [creator, token] = split_cookie(cookie).ok_or(crate::Error::NotLoggedIn)?;
        let creator = creator
            .parse::<i64>()
            .map_err(|_e| crate::Error::NotLoggedIn)?;

        Ok(Self {
            creator,
            token: token.to_string(),
        })
    }
}

impl CreatorSession {
    pub async fn get_from_cookie(db: &mut DbConn, cookie: &str) -> crate::Result<Option<Self>> {
        let cookie = SessionCookie::from_cookie(cookie)?;
        Ok(creator_sessions
            .find((cookie.creator, cookie.token))
            .select(Self::as_select())
            .first(&mut *db)
            .await
            .optional()?)
    }

    pub async fn new_session(db: &mut DbConn, creator: i64) -> crate::Result<Self> {
        let cookie = SessionCookie::new(creator);
        Ok(diesel::insert_into(creator_sessions)
            .values(cookie)
            .returning(Self::as_returning())
            .get_result(db)
            .await?)
    }
}
