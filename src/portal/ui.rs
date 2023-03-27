use rocket::{
    data::Data,
    http::{ContentType, Method},
    route::Handler,
    route::Outcome,
    Request, Route,
    fairing::Fairing,
};
use rocket_dyn_templates::Template;
use rust_embed::RustEmbed;
use std::{ffi::OsStr, path::PathBuf};

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
                .register_template_string(&name, String::from_utf8_lossy(file.data.as_ref()))
                .unwrap();
        }
    })
}

#[cfg(debug_assertions)]
pub fn template_fairing() -> impl Fairing {
    Template::fairing()
}

#[derive(RustEmbed)]
#[folder = "www/"]
struct RawClientFiles;

#[derive(Clone)]
pub struct StaticClientFiles {
    rank: isize,
}

#[cfg(not(debug_assertions))]
pub fn static_files() -> StaticClientFiles {
    StaticClientFiles::default()
}

#[cfg(debug_assertions)]
pub fn static_files() -> rocket::fs::FileServer {
    rocket::fs::FileServer::from("www")
}

impl Default for StaticClientFiles {
  fn default() -> Self {
      Self { rank: 10 }
  }
}

impl StaticClientFiles {
    pub fn rank(mut self, rank: isize) -> Self {
        self.rank = rank;
        self
    }
}

impl From<StaticClientFiles> for Vec<Route> {
    fn from(server: StaticClientFiles) -> Self {
        let mut route = Route::ranked(server.rank, Method::Get, "/<path..>", server);
        route.name = Some("StaticClientFiles".into());
        vec![route]
    }
}

#[async_trait]
impl Handler for StaticClientFiles {
    async fn handle<'r>(&self, req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r> {
        if let Ok(path) = req.segments::<PathBuf>(0..) {
            let content_type = path
                .extension()
                .and_then(OsStr::to_str)
                .and_then(ContentType::from_extension)
                .unwrap_or(ContentType::Bytes);

            if let Some(content) = path.to_str().and_then(RawClientFiles::get) {
                return Outcome::from_or_forward(req, data, (content_type, content.data));
            }
        }

        Outcome::forward(data)
    }
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
