#[macro_use]
extern crate rocket;

mod db;
mod image_meta;
mod into_inner;
mod portal;
mod redirect;
mod cache;

use rocket::{
    figment::providers::{Format, Toml},
    http::Status,
    log::private::warn,
    response::Responder,
    Config,
};
use rocket_db_pools::Database;
use rocket_dyn_templates::Template;
use thiserror::Error;

#[get("/", format = "html")]
fn index() -> Template {
    Template::render("index", ())
}

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

#[rocket::main]
async fn main() -> Result<()> {
    let _ = rocket::custom(Config::figment().join(Toml::file("Secrets.toml").nested()))
        .manage(cache::Cache::default())
        .attach(db::Imagefork::init())
        .attach(db::Imagefork::init_migrations())
        .manage(image_meta::ImageMetadata::default())
        .manage(portal::auth::AuthClient::default())
        .attach(portal::auth::github::fairing())
        .mount("/", portal::auth::github::routes())
        .attach(portal::token::fairing())
        .attach(Template::fairing())
        .mount("/", portal::auth::routes())
        .mount("/", portal::creators::routes())
        .mount("/", routes![index])
        .mount("/redirect", redirect::routes())
        .launch()
        .await?;
    Ok(())
}
