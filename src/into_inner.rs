pub trait IntoInner<T> {
    fn into_inner(self) -> T;
}

impl<T> IntoInner<T> for rocket::serde::json::Json<T> {
    fn into_inner(self) -> T {
        self.into_inner()
    }
}

impl<T> IntoInner<T> for rocket::form::Form<T> {
    fn into_inner(self) -> T {
        self.into_inner()
    }
}
