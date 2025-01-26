use axum::{extract::FromRef, Router};
use axum_login::login_required;
use serde::Deserialize;

use crate::{
    auth::{AuthConfig, Backend},
    db::DbPool,
};

mod auth;
mod creators;
//mod posters;
//mod token;
//mod ui;

#[derive(Deserialize, Clone)]
pub struct PortalConfig {
    pub auth: AuthConfig,
}

#[derive(FromRef, Clone)]
pub struct PortalState {
    db: DbPool,
}

pub fn routes(db: DbPool) -> Router {
    Router::new()
        .nest("/creator", creators::routes())
        .with_state(PortalState { db })
        .route_layer(login_required!(Backend, login_url = "/auth/github"))
        .nest("/auth", auth::routes())
}
