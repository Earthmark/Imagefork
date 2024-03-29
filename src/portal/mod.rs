use rocket::Route;

pub mod auth;
pub mod creators;
pub mod posters;
pub mod token;
pub mod ui;

pub fn routes() -> Vec<Route> {
    let mut routes = Vec::default();
    routes.append(&mut auth::routes());
    routes.append(&mut creators::routes());
    routes.append(&mut posters::routes());
    routes.append(&mut ui::routes());
    routes
}
