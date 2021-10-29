/*
RUST_BACKTRACE=1 RUST_LOG=trace cargo run -p http-api-isahc-client-demo --bin httpbin
*/

use std::error;

use futures_lite::future::block_on;
use http_api_isahc_client::{Client as _, IsahcClient, RetryableClient as _};
use isahc::HttpClient;

fn main() -> Result<(), Box<dyn error::Error>> {
    pretty_env_logger::init();

    block_on(run())
}

async fn run() -> Result<(), Box<dyn error::Error>> {
    let client = IsahcClient::with(HttpClient::builder().cookies().build()?);

    //
    let ip_endpoint = endpoints::IpEndpoint;
    let ret = client.respond_endpoint(&ip_endpoint).await?;
    println!("{:?}", ret);

    //
    let headers_endpoint = endpoints::HeadersEndpoint;
    let req_header_x_foo = "Foo";
    let mut res_header_server = None;
    let ret = client
        .respond_endpoint_with_callback(
            &headers_endpoint,
            |mut req| {
                req.headers_mut()
                    .insert("X-Foo", req_header_x_foo.parse().unwrap());
                req
            },
            |res| {
                res_header_server = res.headers().get("Server").map(|x| x.as_bytes().to_owned());
            },
        )
        .await?;
    println!(
        "res_header_server: {:?}",
        res_header_server.map(String::from_utf8)
    );
    println!("{:?}", ret);

    //
    let cookies_set_endpoint = endpoints::CookiesSetEndpoint;
    client.respond_endpoint(&cookies_set_endpoint).await?;
    println!("cookie_jar: {:?}", client.http_client.cookie_jar());

    //
    let uuid_endpoint = endpoints::UuidEndpoint;
    let ret = client
        .respond_endpoint_until_done_with_callback(
            &uuid_endpoint,
            |req, _retry| req,
            |_res, retry| {
                println!("retry: {:?}", retry);
            },
        )
        .await?;
    println!("{:?}", ret);

    Ok(())
}

pub mod endpoints {
    use std::collections::HashMap;

    use http_api_endpoint::{
        http::Error as HttpError, Body, Endpoint, Request, Response, RetryableEndpoint,
        RetryableEndpointRetry,
    };
    use serde::Deserialize;
    use serde_json::Error as SerdeJsonError;

    //
    pub struct IpEndpoint;
    #[derive(Deserialize, Debug)]
    pub struct IpEndpointResponseBodyJson {
        pub origin: String,
    }

    impl Endpoint for IpEndpoint {
        type RenderRequestError = HttpError;

        type ParseResponseOutput = IpEndpointResponseBodyJson;
        type ParseResponseError = SerdeJsonError;

        fn render_request(&self) -> Result<Request<Body>, Self::RenderRequestError> {
            Request::builder()
                .uri("https://httpbin.org/ip")
                .body(vec![])
        }

        fn parse_response(
            &self,
            response: Response<Body>,
        ) -> Result<Self::ParseResponseOutput, Self::ParseResponseError> {
            serde_json::from_slice::<IpEndpointResponseBodyJson>(response.body())
        }
    }

    //
    pub struct HeadersEndpoint;
    #[derive(Deserialize, Debug)]
    pub struct HeadersEndpointResponseBodyJson {
        pub headers: HashMap<String, String>,
    }

    impl Endpoint for HeadersEndpoint {
        type RenderRequestError = HttpError;

        type ParseResponseOutput = HeadersEndpointResponseBodyJson;
        type ParseResponseError = SerdeJsonError;

        fn render_request(&self) -> Result<Request<Body>, Self::RenderRequestError> {
            Request::builder()
                .uri("https://httpbin.org/headers")
                .body(vec![])
        }

        fn parse_response(
            &self,
            response: Response<Body>,
        ) -> Result<Self::ParseResponseOutput, Self::ParseResponseError> {
            serde_json::from_slice::<HeadersEndpointResponseBodyJson>(response.body())
        }
    }

    //
    pub struct CookiesSetEndpoint;
    impl Endpoint for CookiesSetEndpoint {
        type RenderRequestError = HttpError;

        type ParseResponseOutput = ();
        type ParseResponseError = SerdeJsonError;

        fn render_request(&self) -> Result<Request<Body>, Self::RenderRequestError> {
            Request::builder()
                .uri("https://httpbin.org/cookies/set?foo=bar")
                .body(vec![])
        }

        fn parse_response(
            &self,
            response: Response<Body>,
        ) -> Result<Self::ParseResponseOutput, Self::ParseResponseError> {
            debug_assert!(response.status().as_u16() == 302);
            Ok(())
        }
    }

    //
    pub struct UuidEndpoint;
    #[derive(Deserialize, Debug)]
    pub struct UuidEndpointResponseBodyJson {
        pub uuid: String,
    }

    impl RetryableEndpoint for UuidEndpoint {
        type RetryReason = String;

        const MAX_RETRY_COUNT: usize = 5;

        type RenderRequestError = HttpError;

        type ParseResponseOutput = UuidEndpointResponseBodyJson;
        type ParseResponseError = SerdeJsonError;

        fn render_request(
            &self,
            _retry: Option<&RetryableEndpointRetry<Self::RetryReason>>,
        ) -> Result<Request<Body>, Self::RenderRequestError> {
            Request::builder()
                .uri("https://httpbin.org/uuid")
                .body(vec![])
        }

        fn parse_response(
            &self,
            response: Response<Body>,
            _retry: Option<&RetryableEndpointRetry<Self::RetryReason>>,
        ) -> Result<Result<Self::ParseResponseOutput, Self::RetryReason>, Self::ParseResponseError>
        {
            let json = serde_json::from_slice::<UuidEndpointResponseBodyJson>(response.body())?;

            if json
                .uuid
                .chars()
                .collect::<Vec<_>>()
                .first()
                .cloned()
                .unwrap()
                .is_alphabetic()
            {
                Ok(Ok(json))
            } else {
                Ok(Err(json.uuid))
            }
        }
    }
}
