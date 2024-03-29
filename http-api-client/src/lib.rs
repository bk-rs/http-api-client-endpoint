use core::time::Duration;

pub use async_trait::async_trait;
pub use http_api_client_endpoint::{http, Body, Request, Response};
use http_api_client_endpoint::{Endpoint, RetryableEndpoint, RetryableEndpointRetry};

#[async_trait]
pub trait Client {
    type RespondError: std::error::Error + Send + Sync + 'static;

    async fn respond(&self, request: Request<Body>) -> Result<Response<Body>, Self::RespondError>;

    async fn respond_endpoint<EP>(
        &self,
        endpoint: &EP,
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
    {
        self.respond_endpoint_with_callback(endpoint, |req| req, |_| {})
            .await
    }

    async fn respond_endpoint_with_callback<EP, PreRCB, PostRCB>(
        &self,
        endpoint: &EP,
        pre_request_callback: PreRCB,
        post_request_callback: PostRCB,
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
        self.respond_dyn_endpoint_with_callback(
            endpoint,
            pre_request_callback,
            post_request_callback,
        )
        .await
    }

    async fn respond_dyn_endpoint<RRE, PRO, PRE>(
        &self,
        endpoint: &(dyn Endpoint<
            RenderRequestError = RRE,
            ParseResponseOutput = PRO,
            ParseResponseError = PRE,
        > + Send
              + Sync),
    ) -> Result<PRO, ClientRespondEndpointError<Self::RespondError, RRE, PRE>>
    where
        RRE: std::error::Error + Send + Sync + 'static,
        PRE: std::error::Error + Send + Sync + 'static,
    {
        self.respond_dyn_endpoint_with_callback(endpoint, |req| req, |_| {})
            .await
    }

    async fn respond_dyn_endpoint_with_callback<RRE, PRO, PRE, PreRCB, PostRCB>(
        &self,
        endpoint: &(dyn Endpoint<
            RenderRequestError = RRE,
            ParseResponseOutput = PRO,
            ParseResponseError = PRE,
        > + Send
              + Sync),
        mut pre_request_callback: PreRCB,
        mut post_request_callback: PostRCB,
    ) -> Result<PRO, ClientRespondEndpointError<Self::RespondError, RRE, PRE>>
    where
        RRE: std::error::Error + Send + Sync + 'static,
        PRE: std::error::Error + Send + Sync + 'static,
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
                if retry.count >= endpoint.max_retry_count() {
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
    RE: std::error::Error + Send + Sync + 'static,
    EPRRE: std::error::Error + Send + Sync + 'static,
    EPPRE: std::error::Error + Send + Sync + 'static,
{
    RespondFailed(RE),
    EndpointRenderRequestFailed(EPRRE),
    EndpointParseResponseFailed(EPPRE),
}
impl<RE, EPRRE, EPPRE> core::fmt::Display for ClientRespondEndpointError<RE, EPRRE, EPPRE>
where
    RE: std::error::Error + Send + Sync + 'static,
    EPRRE: std::error::Error + Send + Sync + 'static,
    EPPRE: std::error::Error + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl<RE, EPRRE, EPPRE> std::error::Error for ClientRespondEndpointError<RE, EPRRE, EPPRE>
where
    RE: std::error::Error + Send + Sync + 'static,
    EPRRE: std::error::Error + Send + Sync + 'static,
    EPPRE: std::error::Error + Send + Sync + 'static,
{
}

//
#[derive(Debug)]
pub enum RetryableClientRespondEndpointUntilDoneError<RE, EPRRE, EPPRE>
where
    RE: std::error::Error + Send + Sync + 'static,
    EPRRE: std::error::Error + Send + Sync + 'static,
    EPPRE: std::error::Error + Send + Sync + 'static,
{
    RespondFailed(RE),
    EndpointRenderRequestFailed(EPRRE),
    EndpointParseResponseFailed(EPPRE),
    ReachedMaxRetries,
}
impl<RE, EPRRE, EPPRE> core::fmt::Display
    for RetryableClientRespondEndpointUntilDoneError<RE, EPRRE, EPPRE>
where
    RE: std::error::Error + Send + Sync + 'static,
    EPRRE: std::error::Error + Send + Sync + 'static,
    EPPRE: std::error::Error + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl<RE, EPRRE, EPPRE> std::error::Error
    for RetryableClientRespondEndpointUntilDoneError<RE, EPRRE, EPPRE>
where
    RE: std::error::Error + Send + Sync + 'static,
    EPRRE: std::error::Error + Send + Sync + 'static,
    EPPRE: std::error::Error + Send + Sync + 'static,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{collections::HashMap, io::Error as IoError, panic};

    use futures_executor::block_on;

    #[derive(Clone)]
    struct MyEndpoint;
    impl Endpoint for MyEndpoint {
        type RenderRequestError = IoError;

        type ParseResponseOutput = ();
        type ParseResponseError = IoError;

        fn render_request(&self) -> Result<Request<Body>, Self::RenderRequestError> {
            unimplemented!()
        }

        fn parse_response(
            &self,
            _response: Response<Body>,
        ) -> Result<Self::ParseResponseOutput, Self::ParseResponseError> {
            unreachable!()
        }
    }

    #[derive(Clone)]
    struct MyClient;
    #[async_trait]
    impl Client for MyClient {
        type RespondError = IoError;

        async fn respond(
            &self,
            _request: Request<Body>,
        ) -> Result<Response<Body>, Self::RespondError> {
            unreachable!()
        }
    }

    #[test]
    fn test_respond_dyn_endpoint() {
        let prev_hook = panic::take_hook();
        panic::set_hook(Box::new(|_| {}));
        let ret = panic::catch_unwind(|| {
            block_on(async move {
                let mut map: HashMap<
                    &'static str,
                    Box<
                        dyn Endpoint<
                                RenderRequestError = IoError,
                                ParseResponseOutput = (),
                                ParseResponseError = IoError,
                            > + Send
                            + Sync,
                    >,
                > = HashMap::new();

                let key = "x";
                map.insert(key, Box::new(MyEndpoint));
                let client = MyClient;

                let endpoint = map.get(key).unwrap();
                client.respond_dyn_endpoint(endpoint.as_ref()).await
            })
        });
        panic::set_hook(prev_hook);

        match ret {
            Err(err) => {
                if let Some(s) = err.downcast_ref::<&str>() {
                    assert!(s.contains("not implemented"))
                } else {
                    panic!("{:?}", err)
                }
            }
            err => panic!("{:?}", err),
        }
    }
}
