#[macro_use]
extern crate rocket;

mod cache;
mod config;
mod db;
mod image_meta;
mod portal;
mod redirect;

use config::bind;

use rocket::{
    figment::{
        providers::{Format, Toml},
        Figment,
    },
    http::Status,
    log::private::warn,
    response::Responder,
    Build, Config, Rocket,
};
use rocket_db_pools::Database;
use rocket_dyn_templates::Template;
use rocket_oauth2::OAuth2;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
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

type Result<T> = std::result::Result<T, Error>;

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        warn!("Error: {}", self);
        (Status::InternalServerError, "Internal server error.").respond_to(request)
    }
}

pub fn config() -> Figment {
    Config::figment().join(Toml::file("Secrets.toml").nested())
}

#[launch]
pub fn rocket() -> Rocket<Build> {
    rocket::custom(config())
        .attach(cache::Cache::init())
        .attach(db::Imagefork::init())
        .attach(db::Imagefork::init_migrations())
        .manage(image_meta::ImageMetadata::default())
        .manage(portal::auth::AuthClient::default())
        .attach(OAuth2::<portal::auth::github::GitHub>::fairing("github"))
        .mount("/", portal::auth::github::routes())
        .attach(bind::<portal::token::TokenConfig>())
        .attach(bind::<cache::TokenCacheConfig>())
        .attach(Template::fairing())
        .mount("/", portal::routes())
        .mount("/redirect", redirect::routes())
}

#[cfg(test)]
mod test {
    use std::fmt::Display;

    use super::rocket;
    use rocket::{http::uri::Origin, local::blocking::Client, Build, Rocket, Route};

    pub struct TestRocket(Rocket<Build>);

    impl Default for TestRocket {
        fn default() -> Self {
            Self(rocket())
        }
    }

    impl TestRocket {
        pub fn new<R: Into<Vec<Route>>>(routes: R) -> Self {
            Self::default().mount(routes)
        }
        pub fn mount<R: Into<Vec<Route>>>(self, routes: R) -> Self {
            Self(self.0.mount("/", routes))
        }
        pub fn client(self) -> TestClient {
            TestClient(Client::tracked(self.0).unwrap())
        }
    }

    pub struct TestClient(Client);

    impl TestClient {
        pub fn get_string<'c, 'u, U: TryInto<Origin<'u>> + Display>(&'c self, uri: U) -> String {
            self.0.get(uri).dispatch().into_string().unwrap()
        }
        pub fn get<'c, 'u, U: TryInto<Origin<'u>> + Display>(&'c self, uri: U) {
            self.0.get(uri).dispatch();
        }
        pub fn get_json<
            'c,
            'u,
            T: serde::de::DeserializeOwned + Send + 'static,
            U: TryInto<Origin<'u>> + Display,
        >(
            &'c self,
            uri: U,
        ) -> T {
            self.0.get(uri).dispatch().into_json().unwrap()
        }
        pub fn get_maybe_json<
            'c,
            'u,
            T: serde::de::DeserializeOwned + Send + 'static,
            U: TryInto<Origin<'u>> + Display,
        >(
            &'c self,
            uri: U,
        ) -> Option<T> {
            self.0.get(uri).dispatch().into_json()
        }
    }

    #[test]
    fn client_createable() {
        TestRocket::default().client();
    }
}
