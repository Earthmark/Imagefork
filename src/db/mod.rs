mod creator;
mod poster;

use rocket::{
    fairing::{self, AdHoc, Fairing},
    Build, Rocket,
};
use rocket_db_pools::{sqlx, Database};
use sqlx::migrate;

pub use creator::{Creator, CreatorToken};
pub use poster::Poster;

#[derive(Database)]
#[database("imagefork")]
pub struct Imagefork(pub sqlx::PgPool);

impl Imagefork {
    pub fn init_migrations() -> impl Fairing {
        AdHoc::try_on_ignite("Migrate imagefork", Self::run_migrations)
    }

    async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
        if let Some(db) = Self::fetch(&rocket) {
            if let Err(e) = migrate!().run(&db.0).await {
                warn!("Failed to migrate DB: {}", e);
                Err(rocket)
            } else {
                Ok(rocket)
            }
        } else {
            Err(rocket)
        }
    }
}
