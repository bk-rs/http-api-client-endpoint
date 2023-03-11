pub use http_api_client;
pub use isahc;

use core::time::Duration;

pub use http_api_client::Client;
#[cfg(any(
    feature = "with-sleep-via-tokio",
    feature = "with-sleep-via-async-timer",
    feature = "with-sleep-via-async-io"
))]
pub use http_api_client::RetryableClient;
use http_api_client::{async_trait, Body, Request, Response};
use isahc::{
    config::Configurable as _, AsyncReadResponseExt as _, Error as IsahcError,
    HttpClient as IsahcHttpClient,
};

#[derive(Debug, Clone)]
pub struct IsahcClient {
    pub http_client: IsahcHttpClient,
    pub body_buf_default_capacity: usize,
}

impl IsahcClient {
    pub fn new() -> Result<Self, IsahcError> {
        Ok(Self::with(
            IsahcHttpClient::builder()
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(30))
                .build()?,
        ))
    }

    pub fn with(http_client: IsahcHttpClient) -> Self {
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
            body.len().unwrap_or(self.body_buf_default_capacity as u64) as usize,
        );

        let mut res = Response::from_parts(head, body);
        res.copy_to(&mut body_buf).await?;

        let (head, _) = res.into_parts();
        let res = Response::from_parts(head, body_buf);

        Ok(res)
    }
}

#[cfg(all(
    feature = "with-sleep-via-tokio",
    not(feature = "with-sleep-via-async-timer"),
    not(feature = "with-sleep-via-async-io")
))]
#[async_trait]
impl RetryableClient for IsahcClient {
    async fn sleep(&self, dur: Duration) {
        async_sleep::sleep::<async_sleep::impl_tokio::Sleep>(dur).await;
    }
}

#[cfg(all(
    not(feature = "with-sleep-via-tokio"),
    feature = "with-sleep-via-async-timer",
    not(feature = "with-sleep-via-async-io")
))]
#[async_trait]
impl RetryableClient for IsahcClient {
    async fn sleep(&self, dur: Duration) {
        async_sleep::sleep::<async_sleep::impl_async_timer::PlatformTimer>(dur).await;
    }
}

#[cfg(all(
    not(feature = "with-sleep-via-tokio"),
    not(feature = "with-sleep-via-async-timer"),
    feature = "with-sleep-via-async-io"
))]
#[async_trait]
impl RetryableClient for IsahcClient {
    async fn sleep(&self, dur: Duration) {
        async_sleep::sleep::<async_sleep::impl_async_io::Timer>(dur).await;
    }
}
