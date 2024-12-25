use axum::response::IntoResponse;

pub enum EitherResp<T1, T2> {
    A(T1),
    B(T2),
}

impl<T1, T2> IntoResponse for EitherResp<T1, T2>
where
    T1: IntoResponse,
    T2: IntoResponse,
{
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::A(a) => a.into_response(),
            Self::B(b) => b.into_response(),
        }
    }
}
