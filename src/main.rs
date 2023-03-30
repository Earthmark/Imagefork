#[macro_use]
extern crate rocket;

mod cache;
mod config;
mod db;
mod image;
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
use rocket_oauth2::OAuth2;
use rocket_prometheus::PrometheusMetrics;
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

fn common_server(figment: Figment) -> Rocket<Build> {
    let prometheus = PrometheusMetrics::new();
    prometheus
        .registry()
        .register(Box::new(cache::CACHE_RESOLUTION.clone()))
        .unwrap();
    Rocket::custom(figment)
        .attach(db::Imagefork::init())
        .attach(db::Imagefork::init_migrations())
        .attach(prometheus.clone())
        .mount("/metrics", prometheus)
}

fn redirect_server(base: Rocket<Build>) -> Rocket<Build> {
    base.attach(cache::Cache::init())
        .attach(bind::<cache::TokenCacheConfig>())
        .mount("/redirect", redirect::routes())
}

fn portal_server(base: Rocket<Build>) -> Rocket<Build> {
    base.manage(image_meta::WebImageMetadataAggregator::default())
        .manage(portal::auth::AuthClient::default())
        .attach(OAuth2::<portal::auth::github::GitHub>::fairing("github"))
        .mount("/", portal::auth::github::routes())
        .attach(bind::<portal::token::TokenConfig>())
        .attach(portal::ui::template_fairing())
        .mount("/", portal::ui::static_files())
        .mount("/", portal::routes())
}

#[launch]
pub fn rocket() -> Rocket<Build> {
    let figment = config();
    match figment.extract_inner("server_kind") {
        Ok("redirect") => redirect_server(common_server(figment)),
        Ok("portal") => portal_server(common_server(figment)),
        _ => portal_server(redirect_server(common_server(figment))),
    }
}

#[cfg(test)]
mod test {
    use std::fmt::Display;

    use super::rocket;
    use rocket::{
        http::{uri::Origin, Status},
        local::blocking::Client,
        Build, Rocket, Route,
    };

    pub use crate::db::creator::test::*;
    pub use crate::db::creator_token::test::*;
    pub use crate::portal::auth::test::*;

    pub struct TestRocket(Rocket<Build>);

    impl Default for TestRocket {
        fn default() -> Self {
            Self(
                rocket()
                    .mount("/", crate::portal::auth::test::routes())
                    .mount("/", crate::db::creator_token::test::routes())
                    .mount("/", crate::db::creator::test::routes()),
            )
        }
    }

    impl TestRocket {
        pub fn new<R: Into<Vec<Route>>>(r: R) -> Self {
            Self::default().mount(r)
        }
        pub fn mount<R: Into<Vec<Route>>>(self, r: R) -> Self {
            Self(self.0.mount("/", r))
        }
        pub fn client(self) -> TestClient {
            TestClient(Client::tracked(self.0).unwrap())
        }
    }

    pub struct TestClient(Client);

    impl TestClient {
        pub fn get<'c, 'u, U: TryInto<Origin<'u>> + Display>(&'c self, uri: U) -> Status {
            self.0.get(uri).dispatch().status()
        }
        pub fn get_string<'c, 'u, U: TryInto<Origin<'u>> + Display>(&'c self, uri: U) -> String {
            self.0.get(uri).dispatch().into_string().unwrap()
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
