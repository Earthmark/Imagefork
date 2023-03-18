use rocket::{fairing::Fairing, Build, Rocket};
use serde::Deserialize;
use std::marker::PhantomData;

struct Config<C: Deserialize<'static>> {
    _c: PhantomData<C>,
}

pub trait ConfigInfo {
    fn field() -> &'static str;
    fn name() -> &'static str;
}

pub fn bind<C: ConfigInfo + Deserialize<'static> + Send + Sync + 'static>() -> impl Fairing {
    Config::<C> {
        _c: PhantomData::default(),
    }
}

#[async_trait]
impl<C: ConfigInfo + Deserialize<'static> + Send + Sync + 'static> Fairing for Config<C> {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: C::name(),
            kind: rocket::fairing::Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> rocket::fairing::Result {
        match rocket.figment().extract_inner::<C>(C::field()) {
            Ok(config) => Ok(rocket.manage(config)),
            Err(e) => {
                warn!(
                    "Failed to find config {} at {}: {}",
                    C::name(),
                    C::field(),
                    e
                );
                Err(rocket)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::rocket;
    use rocket::{figment::providers::Serialized, local::blocking::Client, Config};
    use serde::{Deserialize, Serialize};

    use super::ConfigInfo;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestConfig {
        string_val: String,
    }

    impl ConfigInfo for TestConfig {
        fn field() -> &'static str {
            "testconf"
        }

        fn name() -> &'static str {
            "testconf config"
        }
    }

    #[test]
    fn config_routes_to_subfield() {
        let rocket = rocket::Rocket::custom(Config::figment().join(Serialized::global(
            "testconf",
            TestConfig {
                string_val: "tacos".to_string(),
            },
        )))
        .attach(super::bind::<TestConfig>());
        let client = Client::tracked(rocket).expect("valid rocket instance");
        let config = client.rocket().state::<TestConfig>();
        assert_eq!(
            config,
            Some(&TestConfig {
                string_val: "tacos".to_string()
            })
        )
    }
}
