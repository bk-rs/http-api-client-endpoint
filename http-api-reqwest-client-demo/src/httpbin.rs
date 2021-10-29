/*
RUST_BACKTRACE=1 RUST_LOG=trace cargo run -p http-api-reqwest-client-demo --bin httpbin
*/

use std::error;

use http_api_reqwest_client::{Client as _, ReqwestClient};
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    pretty_env_logger::init();

    run().await
}

async fn run() -> Result<(), Box<dyn error::Error>> {
    let client = ReqwestClient::with(Client::builder().use_native_tls().build()?);

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

    Ok(())
}

pub mod endpoints {
    use std::collections::HashMap;

    use http_api_endpoint::{http::Error as HttpError, Body, Endpoint, Request, Response};
    use serde::Deserialize;
    use serde_json::Error as SerdeJsonError;

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
