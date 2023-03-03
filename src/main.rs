#[macro_use]
extern crate rocket;

mod protos;
mod store;

use protos::user::Creative;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::State;
use store::Store;

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

#[post("/creative", format = "json", data = "<creative>")]
fn new_creative(store: &State<Store>, mut creative: Json<Creative>) -> Json<Creative> {
    store.set(&mut creative).unwrap();
    creative
}

#[get("/creative/<id>")]
fn get_creative(store: &State<Store>, id: u64) -> Option<Json<Creative>> {
    store.get(id).unwrap().map(Into::into)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(store::Store::new("data.sled").unwrap())
        .mount("/", routes![index, new_creative, get_creative])
}
