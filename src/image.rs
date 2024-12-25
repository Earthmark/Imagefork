use axum::{http::header, response::IntoResponse};
use mediatype::MediaType;

#[derive(Clone)]
pub struct StaticImage {
    format: &'static MediaType<'static>,
    data: &'static [u8],
}

impl StaticImage {
    pub const fn new(format: &'static MediaType, data: &'static [u8]) -> Self {
        Self { format, data }
    }

    pub fn to_response(&self) -> impl IntoResponse {
        ([(header::CONTENT_TYPE, self.format.to_string())], self.data)
    }
}
