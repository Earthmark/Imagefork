use async_trait::async_trait;
use axum_login::{AuthUser, AuthnBackend, UserId};
use oauth2::{
    basic::BasicClient, AuthorizationCode, CsrfToken, EndpointNotSet, EndpointSet, Scope,
    TokenResponse,
};
use reqwest::Url;
use serde::Deserialize;
use tracing::{info, warn};

use crate::{
    db::{creator::Creator, DbPool},
    reqs::{GithubApi, HttpClient},
};

pub const CSRF_STATE_KEY: &str = "oauth.csrf-state";

impl AuthUser for Creator {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        &self.email_hash
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct AuthConfig {
    github: GithubAuthConfig,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GithubAuthConfig {
    client_id: String,
    client_secret: String,
    redirect_url: Url,
}

type GithubOauthClient =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

#[derive(Clone)]
pub struct Backend {
    db: DbPool,
    github_client: GithubOauthClient,
    http_client: HttpClient<GithubApi>,
}

impl Backend {
    pub fn new(db: DbPool, config: &AuthConfig) -> Self {
        Self {
            db,
            github_client: BasicClient::new(oauth2::ClientId::new(
                config.github.client_id.to_string(),
            ))
            .set_client_secret(oauth2::ClientSecret::new(
                config.github.client_secret.to_string(),
            ))
            .set_auth_uri(
                oauth2::AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
                    .unwrap(),
            )
            .set_token_uri(
                oauth2::TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
                    .unwrap(),
            )
            .set_redirect_uri(oauth2::RedirectUrl::from_url(
                config.github.redirect_url.clone(),
            )),
            http_client: HttpClient::new(GithubApi),
        }
    }

    pub fn authorize_url(&self) -> (Url, CsrfToken) {
        self.github_client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("user:email".to_string()))
            .url()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub code: String,
    pub old_state: CsrfToken,
    pub new_state: CsrfToken,
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = Creator;

    type Credentials = Credentials;

    type Error = crate::Error;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        if creds.old_state.secret() != creds.new_state.secret() {
            warn!("An invalid CSRF token was used.");
            return Ok(None);
        }

        let client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("Building reqwest client.");

        let token = self
            .github_client
            .exchange_code(AuthorizationCode::new(creds.code.clone()))
            .request_async(&client)
            .await
            .unwrap();

        let emails = self
            .http_client
            .user_emails(token.access_token().secret())
            .await?;
        let email = &emails[0].email;

        let creator = if let Some(creator) = Creator::get_by_email(&self.db, email).await? {
            info!("Existing user logged in.");
            creator
        } else {
            info!("Creating new user, as a login was attempted for an unknown email.",);
            Creator::create_by_email(&self.db, email).await?
        };

        Ok(Some(creator))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(sqlx::query_as!(
            Self::User,
            "SELECT id, email_hash FROM creators WHERE id = $1",
            user_id
        )
        .fetch_optional(&self.db)
        .await?)
    }
}

pub type AuthSession = axum_login::AuthSession<Backend>;
