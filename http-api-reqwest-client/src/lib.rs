pub use http_api_client;
pub use reqwest;

use std::time::Duration;

use http_api_client::{
    async_trait, http::response::Builder as HttpResponseBuilder, Body, Request, Response,
};
pub use http_api_client::{Client, RetryableClient};
use reqwest::{Client as ReqwestHttpClient, Error as ReqwestError, Request as ReqwestRequest};

#[derive(Debug, Clone)]
pub struct ReqwestClient {
    http_client: ReqwestHttpClient,
}

impl ReqwestClient {
    pub fn new() -> Result<Self, ReqwestError> {
        Ok(Self::with(
            ReqwestHttpClient::builder()
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(30))
                .build()?,
        ))
    }

    pub fn with(http_client: ReqwestHttpClient) -> Self {
        Self { http_client }
    }
}

#[async_trait]
impl Client for ReqwestClient {
    type RespondError = ReqwestError;

    async fn respond(&self, request: Request<Body>) -> Result<Response<Body>, Self::RespondError> {
        let request = ReqwestRequest::try_from(request)?;

        let res = self.http_client.execute(request).await?;

        let http_res = HttpResponseBuilder::new()
            .status(res.status())
            .version(res.version())
            .body(())
            .unwrap();
        let (mut head, _) = http_res.into_parts();
        head.headers = res.headers().to_owned();

        let body_bytes = res.bytes().await?;

        let res = Response::from_parts(head, body_bytes.to_vec());

        Ok(res)
    }
}

#[async_trait]
impl RetryableClient for ReqwestClient {
    async fn sleep(&self, dur: Duration) {
        tokio::time::sleep(dur).await;
    }
}
