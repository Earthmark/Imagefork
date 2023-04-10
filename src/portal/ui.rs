use crate::db::{CreatorToken, Imagefork, Poster};
use crate::Result;
use rocket::{fairing::Fairing, Route};
use rocket_db_pools::Connection;
use rocket_dyn_templates::{context, Template};

pub fn routes() -> Vec<Route> {
    routes![posters, index]
}

#[get("/", format = "html", rank = 2)]
fn index(token: Option<&CreatorToken>) -> Template {
    Template::render(
        "index",
        context! {
            not_logged_in: token.is_none()
        },
    )
}

#[get("/posters", format = "html", rank = 2)]
pub async fn posters(mut db: Connection<Imagefork>, token: &CreatorToken) -> Result<Template> {
    let posters = Poster::get_all_by_creator(&mut db, token.id).await?;
    Ok(Template::render(
        "posters",
        context! {
            moderator: token.moderator,
            lockout: token.lockout,
            poster_limit: token.poster_limit,
            under_limit: posters.len() < token.poster_limit as usize,
            posters,
        },
    ))
}

pub fn template_fairing() -> impl Fairing {
    Template::fairing()
}

pub fn static_files() -> rocket::fs::FileServer {
    rocket::fs::FileServer::from("www")
}

#[cfg(test)]
mod test {
    use crate::rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn index_returns_web_page() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get(uri!(super::index)).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().unwrap().len() > 10, true);
    }
}
