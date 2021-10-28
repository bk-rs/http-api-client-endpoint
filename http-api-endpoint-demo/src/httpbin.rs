/*
RUST_BACKTRACE=1 RUST_LOG=trace cargo run -p http-api-endpoint-demo --bin httpbin
*/

use std::error;

use futures_lite::future::block_on;
use http_api_isahc_client::{Client as _, IsahcClient};

fn main() -> Result<(), Box<dyn error::Error>> {
    pretty_env_logger::init();

    block_on(run())
}

async fn run() -> Result<(), Box<dyn error::Error>> {
    let client = IsahcClient::new()?;

    let ip_endpoint = endpoints::IpEndpoint;
    let ret = client.respond_endpoint(&ip_endpoint).await?;
    println!("{:?}", ret);

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

    Ok(())
}

pub mod endpoints {
    use std::collections::HashMap;

    use http_api_endpoint::{http::Error as HttpError, Body, Endpoint, Request, Response};
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
}
