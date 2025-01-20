use axum::Router;


use crate::{auth::AuthSession, db::DbConn, Result};

pub fn routes() -> Router {
    Rotuer::new()
}

#[axum::debug_handler(state = super::PortalState)]
async fn get_creator(
    db: DbConn,
    auth_session: AuthSession,
) -> Result<Option<Json<Creator>>> {

}

#[get("/creator", format = "json", rank = 2)]
fn get_creator_no_token() -> Unauthorized<()> {
    
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
