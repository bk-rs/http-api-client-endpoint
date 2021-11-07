use std::{error, fmt, time::Duration};

#[cfg(feature = "with-downcast-endpoint")]
use downcast_rs::{impl_downcast, DowncastSync};
#[cfg(feature = "with-clone-endpoint")]
use dyn_clone::{clone_trait_object, DynClone};
pub use http::{self, Request, Response};

pub type Body = Vec<u8>;
pub const MIME_APPLICATION_JSON: &str = "application/json";

#[cfg(all(feature = "with-clone-endpoint", feature = "with-downcast-endpoint"))]
pub trait Endpoint: DynClone + DowncastSync {
    type RenderRequestError: error::Error + 'static;

    type ParseResponseOutput;
    type ParseResponseError: error::Error + 'static;

    fn render_request(&self) -> Result<Request<Body>, Self::RenderRequestError>;

    fn parse_response(
        &self,
        response: Response<Body>,
    ) -> Result<Self::ParseResponseOutput, Self::ParseResponseError>;
}
#[cfg(all(
    feature = "with-clone-endpoint",
    not(feature = "with-downcast-endpoint")
))]
pub trait Endpoint: DynClone {
    type RenderRequestError: error::Error + 'static;

    type ParseResponseOutput;
    type ParseResponseError: error::Error + 'static;

    fn render_request(&self) -> Result<Request<Body>, Self::RenderRequestError>;

    fn parse_response(
        &self,
        response: Response<Body>,
    ) -> Result<Self::ParseResponseOutput, Self::ParseResponseError>;
}
#[cfg(all(
    not(feature = "with-clone-endpoint"),
    feature = "with-downcast-endpoint"
))]
pub trait Endpoint: DowncastSync {
    type RenderRequestError: error::Error + 'static;

    type ParseResponseOutput;
    type ParseResponseError: error::Error + 'static;

    fn render_request(&self) -> Result<Request<Body>, Self::RenderRequestError>;

    fn parse_response(
        &self,
        response: Response<Body>,
    ) -> Result<Self::ParseResponseOutput, Self::ParseResponseError>;
}
#[cfg(all(
    not(feature = "with-clone-endpoint"),
    not(feature = "with-downcast-endpoint")
))]
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

#[cfg(feature = "with-clone-endpoint")]
clone_trait_object!(<RenderRequestError, ParseResponseOutput, ParseResponseError> Endpoint<RenderRequestError = RenderRequestError, ParseResponseOutput = ParseResponseOutput, ParseResponseError = ParseResponseError>);
#[cfg(feature = "with-downcast-endpoint")]
impl_downcast!(Endpoint assoc RenderRequestError, ParseResponseOutput, ParseResponseError);

impl<RenderRequestError, ParseResponseOutput, ParseResponseError> fmt::Debug
    for dyn Endpoint<
            RenderRequestError = RenderRequestError,
            ParseResponseOutput = ParseResponseOutput,
            ParseResponseError = ParseResponseError,
        > + Send
        + Sync
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Endpoint").finish()
    }
}

pub trait RetryableEndpoint {
    type RetryReason: Send + Sync + Clone;

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

#[cfg(feature = "with-clone-endpoint")]
clone_trait_object!(<RetryReason, RenderRequestError, ParseResponseOutput, ParseResponseError> RetryableEndpoint<RetryReason = RetryReason, RenderRequestError = RenderRequestError, ParseResponseOutput = ParseResponseOutput, ParseResponseError = ParseResponseError>);
#[cfg(feature = "with-downcast-endpoint")]
impl_downcast!(RetryableEndpoint assoc RetryReason, RenderRequestError, ParseResponseOutput, ParseResponseError);

impl<RetryReason, RenderRequestError, ParseResponseOutput, ParseResponseError> fmt::Debug
    for dyn RetryableEndpoint<
            RetryReason = RetryReason,
            RenderRequestError = RenderRequestError,
            ParseResponseOutput = ParseResponseOutput,
            ParseResponseError = ParseResponseError,
        > + Send
        + Sync
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RetryableEndpoint").finish()
    }
}

//
pub struct RetryableEndpointRetry<T> {
    pub count: usize,
    pub reason: T,
}

impl<T> fmt::Debug for RetryableEndpointRetry<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
