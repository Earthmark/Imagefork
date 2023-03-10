use rocket::get;
use rocket::serde::json::Json;
use rocket_okapi::{openapi, openapi_get_routes, JsonSchema};
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};

fn get_docs() -> SwaggerUIConfig {
  use rocket_okapi::settings::UrlObject;

  SwaggerUIConfig {
      url: "/my_resource/openapi.json".to_string(),
      ..Default::default()
  }
}
