mod cache;
mod db;
mod error;
mod image;
//mod image_meta;
//mod auth;
//mod html;
//mod portal;
mod prelude;
mod redirect;
mod reqs;
mod service;
//mod session;

use axum::{routing::get, Router};
use axum_login::AuthManagerLayerBuilder;
use axum_prometheus::PrometheusMetricLayer;
use figment::providers::Format;
use serde::Deserialize;

pub use error::*;
use service::{run_with_ctl_c, Service};
use time::Duration;
use tower_sessions::SessionManagerLayer;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn config() -> AppConfig {
    figment::Figment::new()
        .join(figment::providers::Env::prefixed("APP_"))
        .join(figment::providers::Toml::file("imagefork.toml"))
        .extract()
        .unwrap()
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn run_app(config: &AppConfig) -> Result<()> {
    let (monitoring_layer, monitoring_router) = {
        let (mut prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();
        prometheus_layer.enable_response_body_size();
        let router = Router::new()
            .route(
                "/metrics",
                get(move || std::future::ready(metric_handle.render())),
            )
            .layer(prometheus_layer.clone());

        (prometheus_layer, router)
    };

    let app_router: Result<Router> = {
        let db = db::build_pool(&config.core_pg_url.as_str()).await?;

        let mut included_services = Vec::new();

        let mut router = Router::new();
        if let Some(opts) = &config.redirect {
            included_services.push("redirect");
            let tokens = cache::build_pool(&opts.coherency_token_redis_url).await?;

            router = router.nest(
                "/redirect",
                redirect::create_router(
                    db.clone(),
                    tokens,
                    opts.coherency_token_keepalive_minutes * 60,
                ),
            );
        }

        /*
        if let Some(config) = &config.portal {
            included_services.push("portal");

            let session_store = session::Store::new(db.clone());
            let session_layer = SessionManagerLayer::new(session_store)
                .with_secure(false)
                .with_same_site(tower_sessions::cookie::SameSite::Lax)
                .with_expiry(tower_sessions::Expiry::OnInactivity(Duration::days(1)));

            let backend = auth::Backend::new(db.clone(), &config.auth);
            let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

            router = router.merge(portal::routes(db.clone())).layer(auth_layer);
        }
        */

        if included_services.is_empty() {
            error!("No services were selected to be run in the app binding.");
            return Err(Error::InternalError(InternalError::SystemError(
                "No services were selected to be run in the app binding".into(),
            )));
        }

        info!("Included services: {}", included_services.join(", "));

        Ok(router.layer(monitoring_layer))
    };

    run_with_ctl_c(
        [
            Service::new("app", config.app_addr.clone(), app_router?),
            Service::new(
                "monitoring",
                config.monitoring_addr.clone(),
                monitoring_router,
            ),
        ]
        .into_iter(),
    )
    .await?;

    Ok(())
}

#[derive(Deserialize, Clone)]
struct AppConfig {
    app_addr: String,
    monitoring_addr: String,
    core_pg_url: String,
    redirect: Option<redirect::RedirectConfig>,
    //portal: Option<portal::PortalConfig>,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let config = config();

    run_app(&config).await?;

    Ok(())
}

/*
#[cfg(test)]
mod test {
    use std::fmt::Display;

    use super::rocket;
    use rocket::{
        http::{uri::Origin, Status},
        local::blocking::Client,
        Build, Rocket, Route,
    };

    pub use crate::db::creator::test::*;
    pub use crate::db::creator_token::test::*;
    pub use crate::portal::auth::test::*;

    pub struct TestRocket(Rocket<Build>);

    impl Default for TestRocket {
        fn default() -> Self {
            Self(
                rocket()
                    .mount("/", crate::portal::auth::test::routes())
                    .mount("/", crate::db::creator_token::test::routes())
                    .mount("/", crate::db::creator::test::routes()),
            )
        }
    }

    impl TestRocket {
        pub fn new<R: Into<Vec<Route>>>(r: R) -> Self {
            Self::default().mount(r)
        }
        pub fn mount<R: Into<Vec<Route>>>(self, r: R) -> Self {
            Self(self.0.mount("/", r))
        }
        pub fn client(self) -> TestClient {
            TestClient(Client::tracked(self.0).unwrap())
        }
    }

    pub struct TestClient(Client);

    impl TestClient {
        pub fn get<'c, 'u, U: TryInto<Origin<'u>> + Display>(&'c self, uri: U) -> Status {
            self.0.get(uri).dispatch().status()
        }
        pub fn get_string<'c, 'u, U: TryInto<Origin<'u>> + Display>(&'c self, uri: U) -> String {
            self.0.get(uri).dispatch().into_string().unwrap()
        }
        pub fn get_json<
            'c,
            'u,
            T: serde::de::DeserializeOwned + Send + 'static,
            U: TryInto<Origin<'u>> + Display,
        >(
            &'c self,
            uri: U,
        ) -> T {
            self.0.get(uri).dispatch().into_json().unwrap()
        }
        pub fn get_maybe_json<
            'c,
            'u,
            T: serde::de::DeserializeOwned + Send + 'static,
            U: TryInto<Origin<'u>> + Display,
        >(
            &'c self,
            uri: U,
        ) -> Option<T> {
            self.0.get(uri).dispatch().into_json()
        }
    }

    #[test]
    fn client_createable() {
        TestRocket::default().client();
    }
}
*/
