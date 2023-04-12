use rocket::{http::Status, log::private::warn, response::Responder, serde::json::Json};
use rocket_dyn_templates::{context, Template};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
#[serde(tag = "type")]
pub enum Error {
    #[error("User is not logged in")]
    NotLoggedIn,
    #[error("Operation is not allowed as the user is locked out")]
    LockedOut,
    #[error("Operation is only allowed to admin users")]
    UserNotAdmin,
    #[error("Internal Error: {0}")]
    InternalError(#[serde(skip_serializing)] InternalError),
}

impl Error {
    pub fn internal_from<T: Into<InternalError>>(t: T) -> Self {
        Self::InternalError(t.into())
    }
    fn get_status(&self) -> Status {
        match self {
            Error::NotLoggedIn => Status::Unauthorized,
            Error::UserNotAdmin => Status::Unauthorized,
            Error::LockedOut => Status::Forbidden,
            Error::InternalError(_) => Status::InternalServerError,
        }
    }
    fn get_json_error(&self) -> &'static str {
        match self {
            Error::NotLoggedIn => "User is not logged in, use /login/github to log in.",
            Error::UserNotAdmin => "Admin is required.",
            Error::LockedOut => "Creator is locked out due to moderator review, contact support for assistance.",
            Error::InternalError(_) => "Internal error, please contact the service owner.",
        }
    }
    pub fn with_status(self) -> (Status, Self) {
      (self.get_status(), self)
    }
}

impl<T: Into<InternalError>> From<T> for Error {
    fn from(value: T) -> Self {
        Self::internal_from(value)
    }
}

#[derive(Serialize)]
struct VisibleErrorJson {
    error: &'static str,
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        if let Some(accept) = request.accept() {
            for media in accept.iter() {
                if media.is_html() {
                    return (
                        self.get_status(),
                        Template::render(
                            "error",
                            context! {
                              error: self,
                              hide_login: true,
                            },
                        ),
                    )
                        .respond_to(request);
                } else if media.is_json() {
                    break;
                }
            }
        }
        let json = Json::from(VisibleErrorJson {
            error: self.get_json_error(),
        });
        (self.get_status(), json).respond_to(request)
    }
}

#[derive(Error, Debug)]
pub enum InternalError {
    #[error("Sql: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Rocket: {0}")]
    Rocket(#[from] rocket::Error),
    #[error("Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Serde Json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Redis: {0}")]
    Redis(#[from] rocket_db_pools::deadpool_redis::redis::RedisError),
    #[error("System: {0}")]
    SystemError(String),
}

impl From<&str> for InternalError {
    fn from(value: &str) -> Self {
        Self::SystemError(value.to_string())
    }
}

impl From<String> for InternalError {
    fn from(value: String) -> Self {
        Self::SystemError(value)
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for InternalError {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        warn!("Error: {}", self);
        (Status::InternalServerError, "Internal server error.").respond_to(request)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
