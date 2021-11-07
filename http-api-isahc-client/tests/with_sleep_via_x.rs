use std::io;

use http_api_client_endpoint::{
    Body, Request, Response, RetryableEndpoint, RetryableEndpointRetry,
};

#[derive(Clone)]
struct Foo;
impl RetryableEndpoint for Foo {
    type RetryReason = usize;

    type RenderRequestError = io::Error;

    type ParseResponseOutput = ();
    type ParseResponseError = io::Error;

    fn render_request(
        &self,
        retry: Option<&RetryableEndpointRetry<Self::RetryReason>>,
    ) -> Result<Request<Body>, Self::RenderRequestError> {
        println!("retry: {:?}", retry);

        let url = match rand::random::<u8>() {
            0..=80 => "http://github.com",
            81..=160 => "http://baidu.com",
            _ => "http://httpbin.org/ip",
        };

        Ok(Request::builder().uri(url).body(vec![]).unwrap())
    }

    fn parse_response(
        &self,
        _response: Response<Body>,
        retry: Option<&RetryableEndpointRetry<Self::RetryReason>>,
    ) -> Result<Result<Self::ParseResponseOutput, Self::RetryReason>, Self::ParseResponseError>
    {
        if retry.is_some() {
            Ok(Ok(()))
        } else {
            Ok(Err(0))
        }
    }
}

#[cfg(all(
    feature = "with-sleep-via-tokio",
    not(feature = "with-sleep-via-futures-timer"),
    not(feature = "with-sleep-via-async-io")
))]
mod sleep_via_tokio_tests {
    use super::Foo;

    use tokio::runtime::Runtime;

    use http_api_isahc_client::{IsahcClient, RetryableClient as _};

    #[test]
    fn simple() {
        Runtime::new().unwrap().block_on(async {
            let endpoint = Foo;
            let client = IsahcClient::new().unwrap();
            let _ = client.respond_endpoint_until_done(&endpoint).await;
        })
    }
}

#[cfg(all(
    not(feature = "with-sleep-via-tokio"),
    feature = "with-sleep-via-futures-timer",
    not(feature = "with-sleep-via-async-io")
))]
mod sleep_via_futures_timer_tests {
    use super::Foo;

    use futures_executor::block_on;
    use http_api_isahc_client::{IsahcClient, RetryableClient as _};

    #[test]
    fn simple() {
        block_on(async {
            let endpoint = Foo;
            let client = IsahcClient::new().unwrap();
            let _ = client.respond_endpoint_until_done(&endpoint).await;
        })
    }
}

#[cfg(all(
    not(feature = "with-sleep-via-tokio"),
    not(feature = "with-sleep-via-futures-timer"),
    feature = "with-sleep-via-async-io"
))]
mod sleep_via_async_io_tests {
    use super::Foo;

    use futures_executor::block_on;
    use http_api_isahc_client::{IsahcClient, RetryableClient as _};

    #[test]
    fn simple() {
        block_on(async {
            let endpoint = Foo;
            let client = IsahcClient::new().unwrap();
            let _ = client.respond_endpoint_until_done(&endpoint).await;
        })
    }
}
