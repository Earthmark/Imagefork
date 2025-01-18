use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

use super::{util::generate_token, DbConn};
use crate::schema::crsf_tokens::dsl::crsf_tokens;

#[derive(Queryable, Selectable, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::crsf_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CrsfSession {
    pub token: String,
    pub crsf: String,
    pub creation_time: NaiveDateTime,
}

#[derive(Insertable, Queryable)]
#[diesel(table_name = crate::schema::crsf_tokens)]
struct CrsfCookie {
    token: String,
    crsf: String,
}

impl CrsfSession {
    pub async fn new_session(db: &mut DbConn, crsf: &str) -> crate::Result<Self> {
        Ok(diesel::insert_into(crsf_tokens)
            .values(CrsfCookie {
                token: generate_token(),
                crsf: crsf.to_string(),
            })
            .returning(Self::as_returning())
            .get_result(db)
            .await?)
    }

    pub async fn get_and_destroy_from_cookie(db: &mut DbConn, cookie: &str) -> crate::Result<Self> {
        Ok(diesel::delete(crsf_tokens.find(cookie))
            .returning(Self::as_select())
            .get_result(&mut *db)
            .await?)
    }
}
