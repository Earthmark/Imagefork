use askama::Template;
use axum::response::{Html, IntoResponse};

pub struct HtmlTemplate<T>(pub T);

impl<T> From<T> for HtmlTemplate<T>
where
    T: Template,
{
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> axum::response::Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}
