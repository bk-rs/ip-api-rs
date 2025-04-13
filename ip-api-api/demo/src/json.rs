/*
RUST_BACKTRACE=1 RUST_LOG=trace cargo run -p ip-api-api-demo --bin json -- '8.8.8.8'

RUST_BACKTRACE=1 RUST_LOG=trace cargo run -p ip-api-api-demo --bin json -- '8.8.8.8' 'YOUR_API_KEY'
*/

use std::{env, error};

use http_api_reqwest_client::{Client as _, ReqwestClient};
use ip_api_api::endpoints::json::Json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    pretty_env_logger::init();

    let ip = env::args().nth(1).unwrap();
    let key = env::args().nth(2).map(|x| x.into());

    let client = ReqwestClient::new()?;

    let json = Json::new(ip, key);

    let (res, rate_limit) = client.respond_endpoint(&json).await?;

    println!("{:?}", res);
    println!("{:?}", rate_limit);

    Ok(())
}
