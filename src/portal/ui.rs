use rocket::{fairing::Fairing, Route};
use rocket_dyn_templates::Template;

pub fn routes() -> Vec<Route> {
    routes![index]
}

#[get("/", format = "html")]
fn index() -> Template {
    Template::render("index", ())
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
