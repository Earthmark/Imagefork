use crate::into_inner::IntoInner;
use rocket::{
    data::{Data, FromData, Outcome},
    http::{ContentType, MediaType},
    response::Responder,
    Request,
};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use thiserror::Error;

pub trait WellKnownEncoding {
    fn media_type() -> MediaType;
}

pub struct Or<A, B, T>(pub T, PhantomData<A>, PhantomData<B>);

impl<A, B, T> Deref for Or<A, B, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<A, B, T> DerefMut for Or<A, B, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<A, B, T> From<T> for Or<A, B, T> {
    fn from(value: T) -> Self {
        Self(value, Default::default(), Default::default())
    }
}

impl<A, B, T> IntoInner<T> for Or<A, B, T> {
    fn into_inner(self) -> T {
        self.0
    }
}

fn accepted_by<T: WellKnownEncoding>(req: &Request<'_>) -> bool {
    let expected_type = T::media_type();
    if let Some(accept) = req.accept() {
        for media in accept.media_types() {
            if media == &expected_type {
                return true;
            }
        }
    }
    false
}

impl<
        'r,
        A: WellKnownEncoding + Responder<'r, 'static> + From<T>,
        B: Responder<'r, 'static> + From<T>,
        T,
    > Responder<'r, 'static> for Or<A, B, T>
{
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        if accepted_by::<A>(req) {
            A::from(self.0).respond_to(req)
        } else {
            B::from(self.0).respond_to(req)
        }
    }
}

fn through<TS: IntoInner<TM>, TM, TG: From<TM>>(s: TS) -> TG {
    s.into_inner().into()
}

#[derive(Debug, Error)]
pub enum Error<A: Debug, B: Debug> {
    #[error("A: {0:?}")]
    A(A),
    #[error("B: {0:?}")]
    B(B),
}

fn content_type_matches<T: WellKnownEncoding>(req: &Request<'_>) -> bool {
    req.content_type() == Some(&ContentType(T::media_type()))
}

#[async_trait]
impl<'r, A: WellKnownEncoding + FromData<'r> + IntoInner<T>, B: FromData<'r> + IntoInner<T>, T>
    FromData<'r> for Or<A, B, T>
{
    type Error = Error<A::Error, B::Error>;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        if content_type_matches::<A>(req) {
            A::from_data(req, data)
                .await
                .map(through::<A, T, Self>)
                .map_failure(|(s, e)| (s, Error::A(e)))
        } else {
            B::from_data(req, data)
                .await
                .map(through::<B, T, Self>)
                .map_failure(|(s, e)| (s, Error::B(e)))
        }
    }
}
