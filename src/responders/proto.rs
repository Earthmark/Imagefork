use super::or::WellKnownEncoding;
use crate::into_inner::IntoInner;
use protobuf::Message;
use rocket::{
    data::{Data, FromData, Outcome},
    http::{ContentType, MediaType, Status},
    response::{Responder, Result},
    Request,
};
use std::ops::{Deref, DerefMut};
use thiserror::Error;

pub struct Proto<M>(pub M);

impl<M> From<M> for Proto<M> {
    fn from(value: M) -> Self {
        Self(value)
    }
}

impl<T> IntoInner<T> for Proto<T> {
    fn into_inner(self) -> T {
        self.0
    }
}

impl<M> Deref for Proto<M> {
    type Target = M;

    #[inline(always)]
    fn deref(&self) -> &M {
        &self.0
    }
}

impl<M> DerefMut for Proto<M> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut M {
        &mut self.0
    }
}

impl<M> super::or::WellKnownEncoding for Proto<M> {
    fn media_type() -> MediaType {
        MediaType::new("application", "protobuf")
    }
}

impl<'r, 'o: 'r, M: Message> Responder<'r, 'o> for Proto<M> {
    fn respond_to(self, req: &'r Request<'_>) -> Result<'o> {
        (
            ContentType(Self::media_type()),
            self.0.write_to_bytes().map_err(|err| {
                warn!("Error responding with proto {}", err);
                Status::InternalServerError
            })?,
        )
            .respond_to(req)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Bytes: {0:?}")]
    Bytes(<Vec<u8> as FromData<'static>>::Error),
    #[error("Proto: {0:?}")]
    Proto(protobuf::Error),
}

#[async_trait]
impl<'r, M: Message> FromData<'r> for Proto<M> {
    type Error = Error;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        Vec::<u8>::from_data(req, data)
            .await
            .map_failure(|(s, e)| (s, Error::Bytes(e)))
            .and_then(|buff| match M::parse_from_bytes(&buff) {
                Ok(msg) => Outcome::Success(msg.into()),
                Err(e) => Outcome::Failure((Status::BadRequest, Error::Proto(e))),
            })
    }
}
