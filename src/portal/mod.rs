use rocket::Route;
use rocket_dyn_templates::Template;

pub mod auth;
pub mod creators;
pub mod token;

pub fn routes() -> Vec<Route> {
    let mut routes = routes![index];
    routes.append(&mut auth::routes());
    routes.append(&mut creators::routes());
    routes
}

#[get("/", format = "html")]
fn index() -> Template {
    Template::render("index", ())
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
