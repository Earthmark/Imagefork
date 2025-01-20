use crate::db::CreatorToken;
use crate::Result;
use rocket::response::Redirect;
use rocket::Either;
use rocket::{fairing::Fairing, Route};
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
pub async fn posters<'r>(token: Option<&CreatorToken>) -> Result<Either<Template, Redirect>> {
    Ok(if token.is_some() {
        Either::Left(Template::render(
            "posters",
            context! {
                not_logged_in: token.is_none()
            },
        ))
    } else {
        Either::Right(Redirect::to(uri!(index)))
    })
}

pub fn template_fairing() -> impl Fairing {
    Template::fairing()
}

pub fn static_files() -> rocket::fs::FileServer {
    rocket::fs::FileServer::from("www")
}

/*
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
*/
