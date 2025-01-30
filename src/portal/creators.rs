use axum::{response::IntoResponse, routing::get, Json, Router};
use axum_extra::either::Either;
use serde::Serialize;

use crate::auth::{AuthSession, CreatorToken};

pub fn routes() -> Router<super::PortalState> {
    Router::new().route("/creator", get(get_creator))
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Creator {
    pub id: i64,
    pub moderator: bool,
    pub lockout: bool,
    pub poster_limit: i32,
}

impl Creator {
    pub fn from_token(token: &CreatorToken) -> Self {
        Self {
            id: token.id,
            moderator: token.moderator,
            lockout: token.moderator,
            poster_limit: token.poster_limit,
        }
    }
}

#[axum::debug_handler(state = super::PortalState)]
async fn get_creator(auth_session: AuthSession) -> impl IntoResponse {
    if let Some(user) = &auth_session.user {
        Either::E1(Json(Creator::from_token(user)))
    } else {
        Either::E2(axum::http::StatusCode::UNAUTHORIZED)
    }
}

/*
#[cfg(test)]
mod test {
    use crate::{db::Creator, test::*};
    use rocket::http::StatusClass;

    #[test]
    fn no_creator_logged_in() {
        let client = TestRocket::default().client();
        assert_eq!(
            client.get(uri!(super::get_creator)).class(),
            StatusClass::ClientError
        );
    }

    #[test]
    fn creator_logged_in_gets_self() {
        let client = TestRocket::default().client();
        let user = client.creator("pc1");
        user.login();
        let creator: Creator = client.get_json(uri!(super::get_creator()));
        assert_ne!(creator.id, 0);
        assert_eq!(creator.email, "pc1");
        assert_eq!(creator.lockout, false);
        assert_eq!(creator.moderator, false);
        assert!(creator.poster_limit < 10);
    }
}
*/
