use askama::Template;
use axum::{response::IntoResponse, routing::get, Router};

use crate::{auth::AuthSession, db::DbConn, html::HtmlTemplate};

#[derive(Template)]
#[template(path = "creator.html")]
struct CreatorTemplate<'a> {
    title: &'a str,
    name: &'a str,
    show_login: bool,
    logged_in: bool,
}

pub fn routes() -> Router<super::PortalState> {
    Router::new().route("/", get(get_creator))
}

#[axum::debug_handler(state = super::PortalState)]
async fn get_creator(db: DbConn, auth_session: AuthSession) -> impl IntoResponse {
    HtmlTemplate(CreatorTemplate {
        title: "Creator",
        name: "taco",
        show_login: true,
        logged_in: auth_session.user.is_some(),
    })
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
