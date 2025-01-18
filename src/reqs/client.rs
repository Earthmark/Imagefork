use std::future::Future;
use std::sync::Arc;

use metrics::histogram;
use reqwest::{ClientBuilder, IntoUrl, Method, RequestBuilder, Response, StatusCode};

#[derive(Clone, Debug)]
pub struct HttpClient<T: Api> {
    client: Arc<reqwest::Client>,
    api: T,
}

fn make_client() -> ClientBuilder {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("imagefork-server"),
    );
    ClientBuilder::new().default_headers(headers)
}

pub trait Api {
    fn otel_name(&self) -> &'static str;
    fn update_client(&self, builder: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
        builder
    }
}

impl<T: Api> HttpClient<T> {
    pub fn new(api: T) -> Self {
        Self {
            client: Arc::new(api.update_client(make_client()).build().unwrap()),
            api,
        }
    }

    pub async fn request<U: IntoUrl, O, TRes: Future<Output = reqwest::Result<O>>>(
        &self,
        ctx: &'static str,
        method: Method,
        url: U,
        mutator: impl FnOnce(RequestBuilder) -> RequestBuilder,
        parser: impl FnOnce(Response) -> TRes,
    ) -> crate::Result<O> {
        let start = std::time::Instant::now();
        let result = self.request_inner(method, url, mutator, parser).await;
        let duration = start.elapsed().as_millis() as f64;

        let status = result
            .as_ref()
            .map_or("N/A".to_string(), |(status, _)| status.to_string());

        let result = result.and_then(|(_, next)| next);

        let hist = histogram!("http_client_request_duration", "api" => self.api.otel_name(), "ctx" => ctx, "status" => status);
        hist.record(duration);

        Ok(result?)
    }

    async fn request_inner<U: IntoUrl, O, TRes: Future<Output = reqwest::Result<O>>>(
        &self,
        method: Method,
        url: U,
        mutator: impl FnOnce(RequestBuilder) -> RequestBuilder,
        parser: impl FnOnce(Response) -> TRes,
    ) -> reqwest::Result<(StatusCode, reqwest::Result<O>)> {
        let response = mutator(self.client.request(method.clone(), url))
            .send()
            .await?;
        let status = response.status();
        Ok((status, parser(response).await))
    }
}
