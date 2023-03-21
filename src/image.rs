use rocket::{
    http::{ContentType, MediaType, Status},
    response::Responder,
    Response,
};

#[derive(Clone)]
pub struct Image {
    format: MediaType,
    data: &'static [u8],
}

impl Image {
    pub const fn new(format: MediaType, data: &'static [u8]) -> Self {
        Self { format, data }
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Image {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        Response::build_from((Status::Ok, self.data).respond_to(request)?)
            .header(ContentType(self.format))
            .ok()
    }
}
