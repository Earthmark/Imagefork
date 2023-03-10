use rocket::{
    data::Data,
    http::{ContentType, Method},
    route::Handler,
    route::Outcome,
    Request, Route,
};
use rust_embed::RustEmbed;
use std::{ffi::OsStr, path::PathBuf};

#[derive(RustEmbed)]
#[folder = "client/dist/"]
struct RawClientFiles;

#[derive(Clone)]
pub struct StaticClientFiles {
    rank: isize,
}

impl StaticClientFiles {
    pub fn rank(mut self, rank: isize) -> Self {
        self.rank = rank;
        self
    }
}

impl Default for StaticClientFiles {
    fn default() -> Self {
        Self { rank: 10 }
    }
}

impl From<StaticClientFiles> for Vec<Route> {
    fn from(server: StaticClientFiles) -> Self {
        let mut route = Route::ranked(server.rank, Method::Get, "/<path..>", server);
        route.name = Some("StaticClientFiles".into());
        vec![route]
    }
}

fn accepts_html(req: &Request<'_>) -> bool {
    req.accept()
        .map(|acc| acc.media_types().any(|i| i.is_html()))
        .unwrap_or(false)
}

#[async_trait]
impl Handler for StaticClientFiles {
    async fn handle<'r>(&self, req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r> {
        if let Ok(path) = req.segments::<PathBuf>(0..) {
            let payload = if path == PathBuf::new() && accepts_html(req) {
                (ContentType::HTML, RawClientFiles::get("index.html"))
            } else {
                let content_type = path
                    .extension()
                    .and_then(OsStr::to_str)
                    .and_then(ContentType::from_extension)
                    .unwrap_or(ContentType::Bytes);
                (content_type, path.to_str().and_then(RawClientFiles::get))
            };

            if let (content_type, Some(content)) = payload {
                return Outcome::from_or_forward(req, data, (content_type, content.data));
            }
        }

        Outcome::forward(data)
    }
}
