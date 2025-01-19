use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use oauth2::CsrfToken;
use serde::Deserialize;
use tower_sessions::Session;

use crate::auth::{AuthSession, Credentials, CSRF_STATE_KEY};

pub fn routes() -> Router {
    Router::new()
        .route("/", get(begin))
        .route("/authorize", get(authorize))
}

#[axum::debug_handler]
async fn begin(auth_session: AuthSession, session: Session) -> impl IntoResponse {
    let (auth_url, csrf_token) = auth_session.backend.authorize_url();

    session
        .insert(CSRF_STATE_KEY, csrf_token.secret())
        .await
        .expect("Inserting CRSF into session.");

    Redirect::to(auth_url.as_str())
}

#[derive(Debug, Deserialize)]
struct AuthRequest {
    code: String,
    state: CsrfToken,
}

#[axum::debug_handler]
async fn authorize(
    mut auth_session: AuthSession,
    session: Session,
    Query(AuthRequest {
        code,
        state: new_state,
    }): Query<AuthRequest>,
) -> impl IntoResponse {
    let Ok(Some(old_state)) = session.get(CSRF_STATE_KEY).await else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    let creds = Credentials {
        code,
        old_state,
        new_state,
    };

    let user = match auth_session.authenticate(creds).await {
        Ok(Some(user)) => user,
        Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    Redirect::to("/").into_response()
}
