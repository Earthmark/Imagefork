#[macro_use]
extern crate rocket;

mod client;
mod protos;
mod store;
mod responders;
mod into_inner;

use protos::imagefork::Poster;
use responders::ProtoTextProtoJson;
use rocket::response::Redirect;
use rocket::State;
use store::Store;
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

#[post("/creative", data = "<creative>")]
fn new_creative(store: &State<Store>, mut creative: ProtoTextProtoJson<Poster>) -> ProtoTextProtoJson<Poster> {
    store.set(&mut creative).unwrap();
    creative
}

#[get("/creative/<id>")]
fn get_creative(store: &State<Store>, id: u64) -> Option<ProtoTextProtoJson<Poster>> {
    store.get(id).unwrap().map(Into::into)
}

#[derive(Error, Debug)]
enum Error {
    #[error("Store: {0}")]
    Store(#[from] store::Error),
    #[error("Rocket: {0}")]
    Rocket(#[from] rocket::Error),
}

#[rocket::main]
async fn main() -> Result<(), Error> {
    let _ = rocket::build()
        .manage(store::Store::new("data")?)
        .mount("/", routes![index, new_creative, get_creative])
        .mount("/", client::StaticClientFiles::new())
        .launch()
        .await?;
    Ok(())
}
