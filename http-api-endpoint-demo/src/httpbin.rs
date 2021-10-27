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

    Ok(())
}

pub mod endpoints {
    use http_api_endpoint::{http::Error as HttpError, Body, Endpoint, Request, Response};
    use serde::Deserialize;
    use serde_json::Error as SerdeJsonError;

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
}
