#[macro_use]
extern crate rocket;

mod auth;
mod client;
mod db;
mod image_meta;
mod into_inner;
mod portal;
mod store;

use rocket::{
    figment::providers::{Format, Toml},
    http::Status,
    log::private::warn,
    response::{Redirect, Responder},
    Config,
};
use rocket_db_pools::Database;
use thiserror::Error;

#[get("/render?<width>&<aspect>&<noonce>&<panel_id>&<creative_id>")]
fn index(
    width: i32,
    aspect: f32,
    noonce: Option<i32>,
    panel_id: Option<i32>,
    creative_id: Option<u64>,
) -> Redirect {
    Redirect::to(format!(
        "http://localhost/{}/{}/{}/{}/{}",
        width,
        aspect,
        noonce.unwrap_or_default(),
        panel_id.unwrap_or_default(),
        creative_id.unwrap_or_default()
    ))
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Store: {0}")]
    Store(#[from] store::Error),
    #[error("Sql: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Rocket: {0}")]
    Rocket(#[from] rocket::Error),
    #[error("Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
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
        .attach(store::fairing())
        .attach(db::Imagefork::init())
        .attach(db::Imagefork::init_migrations())
        .manage(image_meta::ImageMetadata::default())
        .manage(auth::AuthClient::default())
        .attach(auth::fairing())
        .mount("/", auth::routes())
        .mount("/", portal::routes())
        .mount("/", routes![index])
        .mount("/", client::StaticClientFiles::default())
        .launch()
        .await?;
    Ok(())
}
