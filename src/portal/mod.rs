use axum::Router;
use serde::Deserialize;

use crate::auth::AuthConfig;

mod auth;
//mod creators;
//mod posters;
//mod token;
//mod ui;

#[derive(Deserialize, Clone)]
pub struct PortalConfig {
    pub auth: AuthConfig,
}

pub fn routes() -> Router {
    Router::new().nest("/auth", auth::routes())
    //let mut routes = Vec::default();
    //routes.append(&mut auth::routes());
    //routes.append(&mut creators::routes());
    //routes.append(&mut posters::routes());
    //routes.append(&mut ui::routes());
    //routes
}
