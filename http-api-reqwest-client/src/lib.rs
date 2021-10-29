pub use http_api_client;
pub use reqwest;

use std::time::Duration;

use http_api_client::{async_trait, Body, Request, Response};
pub use http_api_client::{Client, RetryableClient};
use reqwest::{Client as ReqwestHttpClient, Error as ReqwestError, Request as ReqwestRequest};

#[derive(Debug, Clone)]
pub struct ReqwestClient {
    pub http_client: ReqwestHttpClient,
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
        let res_reqwest = self
            .http_client
            .execute(ReqwestRequest::try_from(request)?)
            .await?;

        let res = Response::new(());
        let (mut head, _) = res.into_parts();
        head.status = res_reqwest.status();
        head.version = res_reqwest.version();
        head.headers = res_reqwest.headers().to_owned();

        let body = res_reqwest.bytes().await?.to_vec();

        let res = Response::from_parts(head, body);

        Ok(res)
    }
}

#[async_trait]
impl RetryableClient for ReqwestClient {
    async fn sleep(&self, dur: Duration) {
        tokio::time::sleep(dur).await;
    }
}
