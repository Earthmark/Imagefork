use std::sync::Arc;

use axum::{
    extract::{FromRef, Query, State},
    http::{header::SET_COOKIE, HeaderMap},
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use axum_extra::{headers, TypedHeader};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use reqwest::Url;
use serde::Deserialize;

use crate::{
    db::{
        creator::Creator, creator_session::CreatorSession, crsf_session::CrsfSession, DbConn,
        DbPool,
    },
    reqs::{GithubApi, HttpClient},
    Result,
};

static COOKIE_NAME: &str = "SESSION";

#[derive(Deserialize, Clone, Debug)]
pub struct GithubAuthConfig {
    client_id: String,
    client_secret: String,
    redirect_url: Url,
}

#[derive(Clone)]
struct GitHubClient(Arc<BasicClient>);

impl AsRef<BasicClient> for GitHubClient {
    fn as_ref(&self) -> &BasicClient {
        &self.0
    }
}

impl GitHubClient {
    fn new(config: &GithubAuthConfig) -> Self {
        Self(Arc::new(
            BasicClient::new(
                ClientId::new(config.client_id.to_string()),
                Some(ClientSecret::new(config.client_secret.to_string())),
                AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).unwrap(),
                Some(
                    TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
                        .unwrap(),
                ),
            )
            .set_redirect_uri(RedirectUrl::from_url(config.redirect_url.clone())),
        ))
    }
}

#[derive(FromRef, Clone)]
struct GithubState {
    db: DbPool,
    oauth_client: GitHubClient,
    github_client: HttpClient<GithubApi>,
}

pub fn routes(db: DbPool, config: &GithubAuthConfig) -> Router {
    Router::new()
        .route("/", get(begin))
        .route("/authorize", get(authorize))
        .with_state(GithubState {
            db,
            oauth_client: GitHubClient::new(config),
            github_client: HttpClient::new(GithubApi),
        })
}

#[axum::debug_handler(state = GithubState)]
async fn begin(
    mut db: DbConn,
    State(oauth_client): State<GitHubClient>,
) -> crate::Result<impl IntoResponse> {
    let (auth_url, csrf_token) = oauth_client
        .0
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("user:email".to_string()))
        .url();

    let token_cookie = CrsfSession::new_session(&mut db, csrf_token.secret()).await?;

    let cookie = format!(
        "{COOKIE_NAME}={}; SameSite=Lax; HttpOnly; Secure; Path=/",
        token_cookie.token
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        cookie.parse().map_err(|e| {
            crate::InternalError::SystemError(format!("Failed to parse cookie: {e}"))
        })?,
    );

    Ok((headers, Redirect::to(auth_url.as_str())))
}

async fn validate_crsf_token(db: &mut DbConn, cookie: &str, token: &str) -> crate::Result<()> {
    if CrsfSession::get_and_destroy_from_cookie(db, cookie)
        .await?
        .crsf
        == token
    {
        Ok(())
    } else {
        Err(crate::Error::NotLoggedIn)
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AuthRequest {
    code: String,
    state: String,
}

#[axum::debug_handler(state = GithubState)]
async fn authorize(
    mut db: DbConn,
    State(oauth_client): State<GitHubClient>,
    State(http_client): State<HttpClient<GithubApi>>,
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
    Query(query): Query<AuthRequest>,
) -> Result<impl IntoResponse> {
    validate_crsf_token(
        &mut db,
        cookies.get(COOKIE_NAME).ok_or(crate::Error::NotLoggedIn)?,
        &query.state,
    )
    .await?;

    let token = oauth_client
        .0
        .exchange_code(AuthorizationCode::new(query.code.clone()))
        .request_async(async_http_client)
        .await
        .unwrap();

    let emails = http_client
        .user_emails(token.access_token().secret())
        .await?;
    let email = &emails[0].email;

    let creator = if let Some(creator) = Creator::get_by_email(&mut db, email).await? {
        creator
    } else {
        Creator::create_by_email(&mut db, email).await?
    };

    let session = CreatorSession::new_session(&mut db, creator.id).await?;

    let cookie = format!(
        "{COOKIE_NAME}={}; SameSite=Lax; HttpOnly; Secure; Path=/",
        session.token
    );
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        cookie.parse().map_err(|_e| crate::Error::NotLoggedIn)?,
    );

    Ok((headers, Redirect::to("/")))
}
