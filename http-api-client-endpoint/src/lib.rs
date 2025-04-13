use core::time::Duration;

#[cfg(feature = "dyn-clone")]
use dyn_clone::{clone_trait_object, DynClone};
pub use http::{self, Request, Response};

pub type Body = Vec<u8>;
pub const MIME_APPLICATION_JSON: &str = "application/json";

//
//
//
#[cfg(feature = "dyn-clone")]
pub trait Endpoint: DynClone {
    type RenderRequestError: std::error::Error + Send + Sync + 'static;

    type ParseResponseOutput;
    type ParseResponseError: std::error::Error + Send + Sync + 'static;

    fn render_request(&self) -> Result<Request<Body>, Self::RenderRequestError>;

    fn parse_response(
        &self,
        response: Response<Body>,
    ) -> Result<Self::ParseResponseOutput, Self::ParseResponseError>;
}

#[cfg(not(feature = "dyn-clone"))]
pub trait Endpoint {
    type RenderRequestError: std::error::Error + Send + Sync + 'static;

    type ParseResponseOutput;
    type ParseResponseError: std::error::Error + Send + Sync + 'static;

    fn render_request(&self) -> Result<Request<Body>, Self::RenderRequestError>;

    fn parse_response(
        &self,
        response: Response<Body>,
    ) -> Result<Self::ParseResponseOutput, Self::ParseResponseError>;
}

#[cfg(feature = "dyn-clone")]
clone_trait_object!(<RenderRequestError, ParseResponseOutput, ParseResponseError> Endpoint<RenderRequestError = RenderRequestError, ParseResponseOutput = ParseResponseOutput, ParseResponseError = ParseResponseError>);

impl<RenderRequestError, ParseResponseOutput, ParseResponseError> core::fmt::Debug
    for dyn Endpoint<
        RenderRequestError = RenderRequestError,
        ParseResponseOutput = ParseResponseOutput,
        ParseResponseError = ParseResponseError,
    >
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Endpoint").finish()
    }
}

impl<RenderRequestError, ParseResponseOutput, ParseResponseError> core::fmt::Debug
    for dyn Endpoint<
            RenderRequestError = RenderRequestError,
            ParseResponseOutput = ParseResponseOutput,
            ParseResponseError = ParseResponseError,
        > + Send
        + Sync
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Endpoint").finish()
    }
}

//
//
//
#[cfg(feature = "dyn-clone")]
pub trait RetryableEndpoint: DynClone {
    type RetryReason: Send + Sync + Clone;

    type RenderRequestError: std::error::Error + Send + Sync + 'static;

    type ParseResponseOutput;
    type ParseResponseError: std::error::Error + Send + Sync + 'static;

    fn render_request(
        &self,
        retry: Option<&RetryableEndpointRetry<Self::RetryReason>>,
    ) -> Result<Request<Body>, Self::RenderRequestError>;

    fn parse_response(
        &self,
        response: Response<Body>,
        retry: Option<&RetryableEndpointRetry<Self::RetryReason>>,
    ) -> Result<Result<Self::ParseResponseOutput, Self::RetryReason>, Self::ParseResponseError>;

    fn max_retry_count(&self) -> usize {
        3
    }

    fn next_retry_in(&self, retry: &RetryableEndpointRetry<Self::RetryReason>) -> Duration {
        match retry.count {
            0..=2 => Duration::from_millis(500),
            _ => Duration::from_secs(1),
        }
    }
}

#[cfg(not(feature = "dyn-clone"))]
pub trait RetryableEndpoint {
    type RetryReason: Send + Sync + Clone;

    type RenderRequestError: std::error::Error + Send + Sync + 'static;

    type ParseResponseOutput;
    type ParseResponseError: std::error::Error + Send + Sync + 'static;

    fn render_request(
        &self,
        retry: Option<&RetryableEndpointRetry<Self::RetryReason>>,
    ) -> Result<Request<Body>, Self::RenderRequestError>;

    fn parse_response(
        &self,
        response: Response<Body>,
        retry: Option<&RetryableEndpointRetry<Self::RetryReason>>,
    ) -> Result<Result<Self::ParseResponseOutput, Self::RetryReason>, Self::ParseResponseError>;

    fn max_retry_count(&self) -> usize {
        3
    }

    fn next_retry_in(&self, retry: &RetryableEndpointRetry<Self::RetryReason>) -> Duration {
        match retry.count {
            0 | 1 | 2 => Duration::from_millis(500),
            _ => Duration::from_secs(1),
        }
    }
}

#[cfg(feature = "dyn-clone")]
clone_trait_object!(<RetryReason, RenderRequestError, ParseResponseOutput, ParseResponseError> RetryableEndpoint<RetryReason = RetryReason, RenderRequestError = RenderRequestError, ParseResponseOutput = ParseResponseOutput, ParseResponseError = ParseResponseError>);

impl<RetryReason, RenderRequestError, ParseResponseOutput, ParseResponseError> core::fmt::Debug
    for dyn RetryableEndpoint<
        RetryReason = RetryReason,
        RenderRequestError = RenderRequestError,
        ParseResponseOutput = ParseResponseOutput,
        ParseResponseError = ParseResponseError,
    >
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RetryableEndpoint").finish()
    }
}

impl<RetryReason, RenderRequestError, ParseResponseOutput, ParseResponseError> core::fmt::Debug
    for dyn RetryableEndpoint<
            RetryReason = RetryReason,
            RenderRequestError = RenderRequestError,
            ParseResponseOutput = ParseResponseOutput,
            ParseResponseError = ParseResponseError,
        > + Send
        + Sync
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RetryableEndpoint").finish()
    }
}

//
pub struct RetryableEndpointRetry<T> {
    pub count: usize,
    pub reason: T,
}

impl<T> core::fmt::Debug for RetryableEndpointRetry<T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RetryableEndpointRetry")
            .field("count", &self.count)
            .field("reason", &self.reason)
            .finish()
    }
}

impl<T> RetryableEndpointRetry<T> {
    pub fn new(count: usize, reason: T) -> Self {
        Self { count, reason }
    }
}
