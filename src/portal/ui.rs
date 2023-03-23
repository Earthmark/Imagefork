use rocket::{fairing::Fairing, Route};
use rocket_dyn_templates::Template;
use rust_embed::RustEmbed;

pub fn routes() -> Vec<Route> {
    routes![index]
}

#[get("/", format = "html")]
fn index() -> Template {
    Template::render("index", ())
}

#[derive(RustEmbed)]
#[folder = "templates/"]
struct StaticTemplates;

#[cfg(not(debug_assertions))]
pub fn template_fairing() -> impl Fairing {
    Template::custom(|e| {
        e.handlebars.clear_templates();
        for file_path in StaticTemplates::iter() {
            let file = StaticTemplates::get(file_path.as_ref()).unwrap();
            let name = file_path.replace(".html.hbs", "");
            e.handlebars
                .register_template_string(&name, String::from_utf8_lossy(file.data.as_ref())).unwrap();
        }
    })
}

#[cfg(debug_assertions)]
pub fn template_fairing() -> impl Fairing {
    Template::fairing()
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
