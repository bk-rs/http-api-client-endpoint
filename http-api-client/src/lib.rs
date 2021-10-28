use std::{error, fmt, time::Duration};

pub use async_trait::async_trait;
pub use http_api_endpoint::{http, Body, Request, Response};
use http_api_endpoint::{Endpoint, RetryableEndpoint, RetryableEndpointRetry};

#[async_trait]
pub trait Client {
    type RespondError: error::Error + 'static;

    async fn respond(&self, request: Request<Body>) -> Result<Response<Body>, Self::RespondError>;

    async fn respond_endpoint<EP: Endpoint + Send + Sync>(
        &self,
        endpoint: &EP,
    ) -> Result<
        EP::ParseResponseOutput,
        ClientRespondEndpointError<
            Self::RespondError,
            EP::RenderRequestError,
            EP::ParseResponseError,
        >,
    > {
        self.respond_endpoint_with_callback(endpoint, |req| req, |_| {})
            .await
    }

    async fn respond_endpoint_with_callback<EP, PreRCB, PostRCB>(
        &self,
        endpoint: &EP,
        mut pre_request_callback: PreRCB,
        mut post_request_callback: PostRCB,
    ) -> Result<
        EP::ParseResponseOutput,
        ClientRespondEndpointError<
            Self::RespondError,
            EP::RenderRequestError,
            EP::ParseResponseError,
        >,
    >
    where
        EP: Endpoint + Send + Sync,
        PreRCB: FnMut(Request<Body>) -> Request<Body> + Send,
        PostRCB: FnMut(&Response<Body>) + Send,
    {
        let request = endpoint
            .render_request()
            .map_err(ClientRespondEndpointError::EndpointRenderRequestFailed)?;

        let request = pre_request_callback(request);

        let response = self
            .respond(request)
            .await
            .map_err(ClientRespondEndpointError::RespondFailed)?;

        post_request_callback(&response);

        endpoint
            .parse_response(response)
            .map_err(ClientRespondEndpointError::EndpointParseResponseFailed)
    }
}

#[async_trait]
pub trait RetryableClient: Client {
    async fn sleep(&self, dur: Duration);

    async fn respond_endpoint_until_done<EP>(
        &self,
        endpoint: &EP,
    ) -> Result<
        EP::ParseResponseOutput,
        RetryableClientRespondEndpointUntilDoneError<
            Self::RespondError,
            EP::RenderRequestError,
            EP::ParseResponseError,
        >,
    >
    where
        EP: RetryableEndpoint + Send + Sync,
    {
        self.respond_endpoint_until_done_with_callback(endpoint, |req, _| req, |_, _| {})
            .await
    }

    async fn respond_endpoint_until_done_with_callback<EP, PreRCB, PostRCB>(
        &self,
        endpoint: &EP,
        mut pre_request_callback: PreRCB,
        mut post_request_callback: PostRCB,
    ) -> Result<
        EP::ParseResponseOutput,
        RetryableClientRespondEndpointUntilDoneError<
            Self::RespondError,
            EP::RenderRequestError,
            EP::ParseResponseError,
        >,
    >
    where
        EP: RetryableEndpoint + Send + Sync,
        PreRCB: FnMut(Request<Body>, Option<&RetryableEndpointRetry<EP::RetryReason>>) -> Request<Body>
            + Send,
        PostRCB: FnMut(&Response<Body>, Option<&RetryableEndpointRetry<EP::RetryReason>>) + Send,
    {
        let mut retry = None;

        loop {
            let request = endpoint.render_request(retry.as_ref()).map_err(
                RetryableClientRespondEndpointUntilDoneError::EndpointRenderRequestFailed,
            )?;

            let request = pre_request_callback(request, retry.as_ref());

            let response = self
                .respond(request)
                .await
                .map_err(RetryableClientRespondEndpointUntilDoneError::RespondFailed)?;

            post_request_callback(&response, retry.as_ref());

            match endpoint.parse_response(response, retry.as_ref()).map_err(
                RetryableClientRespondEndpointUntilDoneError::EndpointParseResponseFailed,
            )? {
                Ok(output) => return Ok(output),
                Err(reason) => {
                    let x = retry.get_or_insert(RetryableEndpointRetry::new(0, reason.clone()));
                    x.count += 1;
                    x.reason = reason;
                }
            }

            //
            if let Some(retry) = &retry {
                if retry.count >= EP::MAX_RETRY_COUNT {
                    return Err(RetryableClientRespondEndpointUntilDoneError::ReachedMaxRetries);
                }

                self.sleep(endpoint.next_retry_in(retry)).await;
            }
        }
    }
}

//
#[derive(Debug)]
pub enum ClientRespondEndpointError<RE, EPRRE, EPPRE>
where
    RE: error::Error + 'static,
    EPRRE: error::Error + 'static,
    EPPRE: error::Error + 'static,
{
    RespondFailed(RE),
    EndpointRenderRequestFailed(EPRRE),
    EndpointParseResponseFailed(EPPRE),
}
impl<RE, EPRRE, EPPRE> fmt::Display for ClientRespondEndpointError<RE, EPRRE, EPPRE>
where
    RE: error::Error + 'static,
    EPRRE: error::Error + 'static,
    EPPRE: error::Error + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl<RE, EPRRE, EPPRE> error::Error for ClientRespondEndpointError<RE, EPRRE, EPPRE>
where
    RE: error::Error + 'static,
    EPRRE: error::Error + 'static,
    EPPRE: error::Error + 'static,
{
}

//
#[derive(Debug)]
pub enum RetryableClientRespondEndpointUntilDoneError<RE, EPRRE, EPPRE>
where
    RE: error::Error + 'static,
    EPRRE: error::Error + 'static,
    EPPRE: error::Error + 'static,
{
    RespondFailed(RE),
    EndpointRenderRequestFailed(EPRRE),
    EndpointParseResponseFailed(EPPRE),
    ReachedMaxRetries,
}
impl<RE, EPRRE, EPPRE> fmt::Display
    for RetryableClientRespondEndpointUntilDoneError<RE, EPRRE, EPPRE>
where
    RE: error::Error + 'static,
    EPRRE: error::Error + 'static,
    EPPRE: error::Error + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl<RE, EPRRE, EPPRE> error::Error
    for RetryableClientRespondEndpointUntilDoneError<RE, EPRRE, EPPRE>
where
    RE: error::Error + 'static,
    EPRRE: error::Error + 'static,
    EPPRE: error::Error + 'static,
{
}
