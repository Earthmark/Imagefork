use super::or::WellKnownEncoding;
use crate::into_inner::IntoInner;
use protobuf::MessageFull;
use rocket::{
    data::{Data, FromData, Outcome},
    http::{ContentType, MediaType, Status},
    response::{Responder, Result},
    Request,
};
use std::ops::{Deref, DerefMut};
use thiserror::Error;

pub struct TextProto<M>(pub M);

impl<M> From<M> for TextProto<M> {
    fn from(value: M) -> Self {
        Self(value)
    }
}

impl<T> IntoInner<T> for TextProto<T> {
    fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for TextProto<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for TextProto<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<M> super::or::WellKnownEncoding for TextProto<M> {
    fn media_type() -> MediaType {
        MediaType::new("application", "textproto")
    }
}

impl<'r, 'o: 'r, M: MessageFull> Responder<'r, 'o> for TextProto<M> {
    fn respond_to(self, req: &'r Request<'_>) -> Result<'o> {
        (
            ContentType(Self::media_type()),
            if req.headers().contains("pretty") {
                protobuf::text_format::print_to_string_pretty(&self.0)
            } else {
                protobuf::text_format::print_to_string(&self.0)
            },
        )
            .respond_to(req)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Text: {0:?}")]
    Text(<String as FromData<'static>>::Error),
    #[error("Textproto: {0:?}")]
    TextProto(protobuf::text_format::ParseError),
}

#[async_trait]
impl<'r, M: MessageFull> FromData<'r> for TextProto<M> {
    type Error = Error;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        String::from_data(req, data)
            .await
            .map_failure(|(s, e)| (s, Error::Text(e)))
            .and_then(
                |text| match protobuf::text_format::parse_from_str::<M>(&text) {
                    Ok(msg) => Outcome::Success(msg.into()),
                    Err(e) => Outcome::Failure((Status::BadRequest, Error::TextProto(e))),
                },
            )
    }
}
