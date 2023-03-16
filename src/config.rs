use rocket::{fairing::Fairing, Build, Rocket};
use serde::Deserialize;
use std::marker::PhantomData;

struct Config<C: Deserialize<'static>> {
  field: &'static str,
    _c: PhantomData<C>,
}

pub fn bind<C: Deserialize<'static> + Send + Sync + 'static>(field: &'static str) -> impl Fairing {
  Config::<C> {
    field,
    _c: PhantomData::default(),
  }
}

#[async_trait]
impl<C: Deserialize<'static> + Send + Sync + 'static> Fairing for Config<C> {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: std::any::type_name::<C>(),
            kind: rocket::fairing::Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> rocket::fairing::Result {
        match rocket.figment().extract_inner::<C>(self.field) {
            Ok(config) => Ok(rocket.manage(config)),
            Err(e) => {
                warn!("Failed to find config {}: {}", self.field, e);
                Err(rocket)
            }
        }
    }
}
