use sha2::{Digest, Sha256};
use std::{fmt::Write, time::Duration};
use thiserror::Error;

use reqwest::{
    header,
    header::{HeaderMap, HeaderValue},
    redirect::Policy,
    Client,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Request: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Image: {0}")]
    Image(#[from] image::ImageError),
}

pub struct ImageMetadata {
    pub height: u32,
    pub width: u32,
    pub hash: String,
}

pub struct WebImageMetadataAggregator(Client);

impl Default for WebImageMetadataAggregator {
    fn default() -> Self {
        let mut headers = HeaderMap::new();
        headers.append(header::ACCEPT, HeaderValue::from_static("*/*"));
        headers.append(
            header::USER_AGENT,
            HeaderValue::from_static("Earthmark-Imagefork"),
        );
        Self(
            Client::builder()
                .redirect(Policy::limited(0))
                .connect_timeout(Duration::from_secs(10))
                .default_headers(headers)
                .build()
                .unwrap(),
        )
    }
}

fn get_size(b: &[u8]) -> Result<(u32, u32), Error> {
    let image = image::load_from_memory(b)?;
    Ok((image.height(), image.width()))
}

fn get_hash(b: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b);
    let result = hasher.finalize();
    let mut s = String::with_capacity(2 * result.len());
    for byte in result {
        write!(s, "{:02X}", byte).unwrap();
    }
    s
}

impl WebImageMetadataAggregator {
    // Make use of this in the future once it can be mocked, and we have a cron job to verify metadata.
    #[allow(dead_code)]
    pub async fn get_metadata(&self, url: &str) -> Result<ImageMetadata, Error> {
        let request = self.0.get(url).send().await?;
        let body = request.bytes().await?.to_vec();

        let (height, width) = get_size(&body)?;
        let hash = get_hash(&body);

        Ok(ImageMetadata {
            height,
            width,
            hash,
        })
    }
}
