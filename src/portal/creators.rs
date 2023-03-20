use crate::db::Creator;
use crate::db::CreatorToken;
use crate::db::Imagefork;
use rocket::response::status::Unauthorized;
use rocket::serde::json::Json;
use rocket_db_pools::Connection;

use crate::Result;

pub fn routes() -> Vec<rocket::Route> {
    routes![get_creator, get_creator_no_token,]
}

#[get("/creator", format = "json")]
async fn get_creator(
    mut db: Connection<Imagefork>,
    token: &CreatorToken,
) -> Result<Option<Json<Creator>>> {
    Ok(Creator::get(&mut db, token.id).await?.map(Into::into))
}

#[get("/creator", format = "json", rank = 2)]
fn get_creator_no_token() -> Unauthorized<()> {
    Unauthorized(None)
}

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
