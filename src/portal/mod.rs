use axum::{extract::FromRef, Router};
use serde::Deserialize;

use crate::db::DbPool;

mod auth;
//mod creators;
//mod posters;
//mod token;
//mod ui;

#[derive(Deserialize, Clone)]
pub struct PortalConfig {
    auth: auth::AuthConfig,
}

#[derive(FromRef)]
struct PortalState {
    db: DbPool,
}

pub fn routes(db: DbPool, config: &PortalConfig) -> Router {
    Router::new().nest("/auth", auth::routes(db, &config.auth))
    //let mut routes = Vec::default();
    //routes.append(&mut auth::routes());
    //routes.append(&mut creators::routes());
    //routes.append(&mut posters::routes());
    //routes.append(&mut ui::routes());
    //routes
}
