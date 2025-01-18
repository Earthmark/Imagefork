use super::client::{Api, HttpClient};
use serde::Deserialize;

#[derive(Clone)]
pub struct GithubApi;

impl Api for GithubApi {
    fn otel_name(&self) -> &'static str {
        "github"
    }
}

#[derive(Deserialize, Debug)]
pub struct EmailRecord {
    pub email: String,
}

impl HttpClient<GithubApi> {
    pub async fn user_emails(&self, access_token: &str) -> crate::Result<Vec<EmailRecord>> {
        self.request(
            "user_emails",
            reqwest::Method::GET,
            "https://api.github.com/user/emails",
            |builder| builder.bearer_auth(access_token),
            |response| async { Ok(response.json::<Vec<EmailRecord>>().await?) },
        )
        .await
    }
}
