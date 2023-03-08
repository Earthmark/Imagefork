#[macro_use]
extern crate rocket;

mod client;
mod image_meta;
mod into_inner;
mod portal;
mod store;

use rocket::{fairing::AdHoc, response::Redirect};
use rocket_db_pools::Database;
use rocket_oauth2::OAuth2;
use serde::{Deserialize, Serialize};
use std::result::Result;
use thiserror::Error;

#[derive(Deserialize, Serialize)]
pub struct Poster {
    id: u64,
}

#[derive(Deserialize, Serialize)]
pub struct Creator {
    id: u32,
}

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
enum Error {
    #[error("Store: {0}")]
    Store(#[from] store::Error),
    #[error("Rocket: {0}")]
    Rocket(#[from] rocket::Error),
}

#[derive(Deserialize)]
struct AppConfig {
    url: String,
}

struct Github;

#[rocket::main]
async fn main() -> Result<(), Error> {
    let builder = rocket::build();
    let config = builder
        .figment()
        .focus("databases")
        .extract_inner::<AppConfig>("redirects")
        .expect("Sled config");

    let _ = builder
        .manage(store::Store::new(&config.url)?)
        .attach(portal::Imagefork::init())
        .manage(image_meta::ImageMetadata::default())
        .attach(AdHoc::try_on_ignite(
            "Migrate ImageFork",
            portal::Imagefork::run_migrations,
        ))
        .attach(OAuth2::<Github>::fairing("github"))
        .mount(
            "/",
            routes![
                index,
                portal::get_creator,
                portal::new_creator,
                portal::get_poster,
                portal::new_poster
            ],
        )
        .mount("/", client::StaticClientFiles::new())
        .launch()
        .await?;
    Ok(())
}
