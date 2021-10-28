use std::{error, fmt, time::Duration};

pub use http::{self, Request, Response};

pub type Body = Vec<u8>;
pub const MIME_APPLICATION_JSON: &str = "application/json";

pub trait Endpoint {
    type RenderRequestError: error::Error + 'static;

    type ParseResponseOutput;
    type ParseResponseError: error::Error + 'static;

    fn render_request(&self) -> Result<Request<Body>, Self::RenderRequestError>;

    fn parse_response(
        &self,
        response: Response<Body>,
    ) -> Result<Self::ParseResponseOutput, Self::ParseResponseError>;
}

pub trait RetryableEndpoint {
    type RetryReason: Send + Sync + Clone + fmt::Debug;
    const MAX_RETRY_COUNT: usize = 3;

    type RenderRequestError: error::Error + 'static;

    type ParseResponseOutput;
    type ParseResponseError: error::Error + 'static;

    fn render_request(
        &self,
        retry: Option<&RetryableEndpointRetry<Self::RetryReason>>,
    ) -> Result<Request<Body>, Self::RenderRequestError>;

    fn parse_response(
        &self,
        response: Response<Body>,
        retry: Option<&RetryableEndpointRetry<Self::RetryReason>>,
    ) -> Result<Result<Self::ParseResponseOutput, Self::RetryReason>, Self::ParseResponseError>;

    fn next_retry_in(&self, retry: &RetryableEndpointRetry<Self::RetryReason>) -> Duration {
        match retry.count {
            0 | 1 | 2 => Duration::from_millis(500),
            _ => Duration::from_secs(1),
        }
    }
}

#[derive(Debug)]
pub struct RetryableEndpointRetry<T> {
    pub count: usize,
    pub reason: T,
}
impl<T> RetryableEndpointRetry<T> {
    pub fn new(count: usize, reason: T) -> Self {
        Self { count, reason }
    }
}
