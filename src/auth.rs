use async_trait::async_trait;
use axum_login::{AuthUser, AuthnBackend, UserId};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthorizationCode, CsrfToken, Scope,
    TokenResponse,
};
use reqwest::Url;
use serde::Deserialize;

use crate::{
    db::{creator::Creator, DbConn, DbPool},
    reqs::{GithubApi, HttpClient},
    schema::creators::dsl,
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

#[derive(Clone)]
pub struct Backend {
    db: DbPool,
    github_client: BasicClient,
    http_client: HttpClient<GithubApi>,
}

impl Backend {
    pub fn new(db: DbPool, config: &AuthConfig) -> Self {
        Self {
            db,
            github_client: BasicClient::new(
                oauth2::ClientId::new(config.github.client_id.to_string()),
                Some(oauth2::ClientSecret::new(
                    config.github.client_secret.to_string(),
                )),
                oauth2::AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
                    .unwrap(),
                Some(
                    oauth2::TokenUrl::new(
                        "https://github.com/login/oauth/access_token".to_string(),
                    )
                    .unwrap(),
                ),
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
            return Ok(None);
        }

        let token = self
            .github_client
            .exchange_code(AuthorizationCode::new(creds.code.clone()))
            .request_async(async_http_client)
            .await
            .unwrap();

        let emails = self
            .http_client
            .user_emails(token.access_token().secret())
            .await?;
        let email = &emails[0].email;

        let db = &mut DbConn::from_pool(&self.db).await?;

        let creator = if let Some(creator) = Creator::get_by_email(db, email).await? {
            creator
        } else {
            Creator::create_by_email(db, email).await?
        };

        Ok(Some(creator))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let db = &mut DbConn::from_pool(&self.db).await?;
        Ok(dsl::creators
            .filter(dsl::id.eq(user_id))
            .select(Creator::as_select())
            .get_result(db)
            .await
            .optional()?)
    }
}

pub type AuthSession = axum_login::AuthSession<Backend>;
