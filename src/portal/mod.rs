mod auth;
mod routes;

use rocket::http::Status;
use rocket::response::Responder;
use rocket_db_pools::sqlx;
use thiserror::Error;

pub use routes::routes;

type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Sql: {0}")]
    Sql(#[from] sqlx::Error),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        (Status::InternalServerError, self).respond_to(req)
    }
}
