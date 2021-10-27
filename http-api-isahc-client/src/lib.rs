pub use http_api_client;
pub use isahc;

use std::time::Duration;

pub use http_api_client::Client;
#[cfg(any(
    feature = "sleep-via-tokio",
    feature = "sleep-via-futures-timer",
    feature = "sleep-via-async-io"
))]
pub use http_api_client::RetryableClient;
use http_api_client::{async_trait, Body, Request, Response};
use isahc::{
    config::Configurable as _, AsyncReadResponseExt as _, Error as IsahcError, HttpClient,
};

pub struct IsahcClient {
    http_client: HttpClient,
    pub body_buf_default_capacity: usize,
}

impl IsahcClient {
    pub fn new() -> Result<Self, isahc::Error> {
        Ok(Self::with(
            HttpClient::builder()
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(30))
                .build()?,
        ))
    }

    pub fn with(http_client: HttpClient) -> Self {
        Self {
            http_client,
            body_buf_default_capacity: 4 * 1024,
        }
    }
}

#[async_trait]
impl Client for IsahcClient {
    type RespondError = IsahcError;

    async fn respond(&self, request: Request<Body>) -> Result<Response<Body>, Self::RespondError> {
        let res = self.http_client.send_async(request).await?;
        let (head, body) = res.into_parts();

        let mut body_buf = Vec::with_capacity(
            body.len()
                .unwrap_or_else(|| self.body_buf_default_capacity as u64) as usize,
        );

        let mut res = Response::from_parts(head, body);
        res.copy_to(&mut body_buf).await?;

        let (head, _) = res.into_parts();
        let res = Response::from_parts(head, body_buf);

        Ok(res)
    }
}

#[cfg(all(
    feature = "sleep-via-tokio",
    not(feature = "sleep-via-futures-timer"),
    not(feature = "sleep-via-async-io")
))]
#[async_trait]
impl RetryableClient for IsahcClient {
    async fn sleep(&self, dur: Duration) {
        tokio::time::sleep(dur).await;
    }
}

#[cfg(all(
    not(feature = "sleep-via-tokio"),
    feature = "sleep-via-futures-timer",
    not(feature = "sleep-via-async-io")
))]
#[async_trait]
impl RetryableClient for IsahcClient {
    async fn sleep(&self, dur: Duration) {
        futures_timer::Delay::new(dur).await;
    }
}

#[cfg(all(
    not(feature = "sleep-via-tokio"),
    not(feature = "sleep-via-futures-timer"),
    feature = "sleep-via-async-io"
))]
#[async_trait]
impl RetryableClient for IsahcClient {
    async fn sleep(&self, dur: Duration) {
        async_io::Timer::after(dur).await;
    }
}
