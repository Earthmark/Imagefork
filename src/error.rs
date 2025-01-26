use std::{fmt::Display, sync::Arc};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
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

impl<InnerSrc> From<InnerSrc> for Error
where
    InternalError: From<InnerSrc>,
{
    fn from(value: InnerSrc) -> Self {
        Self::InternalError(value.into())
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (self.get_status(), self.get_json_error()).into_response()
    }
}

impl Error {
    pub fn internal_from<T: Into<InternalError>>(t: T) -> Self {
        Self::InternalError(t.into())
    }
    fn get_status(&self) -> StatusCode {
        match self {
            Error::NotLoggedIn => StatusCode::UNAUTHORIZED,
            Error::UserNotAdmin => StatusCode::UNAUTHORIZED,
            Error::LockedOut => StatusCode::FORBIDDEN,
            Error::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn get_json_error(&self) -> &'static str {
        match self {
            Error::NotLoggedIn => "User is not logged in, use /login/github to log in.",
            Error::UserNotAdmin => "Admin is required.",
            Error::LockedOut => {
                "Creator is locked out due to moderator review, contact support for assistance."
            }
            Error::InternalError(_) => "Internal error, please contact the service owner.",
        }
    }
    pub fn with_status(self) -> (StatusCode, Self) {
        (self.get_status(), self)
    }
    pub fn counter_error_kind(&self) -> &'static str {
        match self {
            Error::NotLoggedIn => "not_logged_in",
            Error::UserNotAdmin => "not_admin",
            Error::LockedOut => "locked_out",
            Error::InternalError(e) => e.counter_error_kind(),
        }
    }
}

#[derive(Error, Debug)]
pub enum InternalError {
    #[error(transparent)]
    Tcp(#[from] std::io::Error),
    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),
    #[error(transparent)]
    DieselInit(#[from] diesel_async::pooled_connection::PoolError),
    #[error(transparent)]
    DieselPool(#[from] diesel_async::pooled_connection::bb8::RunError),
    #[error(transparent)]
    Redis(#[from] bb8_redis::redis::RedisError),
    #[error(transparent)]
    RedisPool(#[from] bb8::RunError<bb8_redis::redis::RedisError>),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    ReqwestMiddleware(#[from] reqwest_middleware::Error),
    //#[error(transparent)]
    //OAuth(#[from] oauth2::basic::BasicRequestTokenError<_>),
    #[error(transparent)]
    AxumLogin(#[from] Arc<axum_login::Error<crate::auth::Backend>>),
    #[error("System: {0}")]
    SystemError(String),
}

impl InternalError {
    pub fn system_error(prefix: impl Display) -> impl FnOnce(&str) -> Self {
        move |context| Self::SystemError(format!("Internal error [{prefix}]: {context}"))
    }

    pub fn counter_error_kind(&self) -> &'static str {
        match self {
            InternalError::Tcp(_) => "tcp",
            InternalError::Diesel(_) => "diesel",
            InternalError::DieselInit(_) => "diesel-init",
            InternalError::DieselPool(_) => "diesel-pool",
            InternalError::Redis(_) => "redis",
            InternalError::RedisPool(_) => "redis-pool",
            InternalError::Reqwest(_) => "reqwest",
            InternalError::ReqwestMiddleware(_) => "reqwest-middleware",
            InternalError::SystemError(_) => "unknown",
            InternalError::AxumLogin(_) => "login",
        }
    }
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

pub type Result<T> = std::result::Result<T, Error>;
