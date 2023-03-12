use rocket::response::Redirect;

#[get("/?<width>&<aspect>&<token>&<panel>")]
pub fn handler(
    width: i32,
    aspect: f32,
    token: Option<i32>,
    panel: Option<i32>,
) -> Redirect {
    Redirect::to(format!(
        "http://localhost/{}/{}/{}/{}",
        width,
        aspect,
        token.unwrap_or_default(),
        panel.unwrap_or_default(),
    ))
}
