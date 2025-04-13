/*
RUST_BACKTRACE=1 RUST_LOG=trace cargo run -p ip-api-api-demo --bin batch -- '8.8.8.8,4.4.4.4'

RUST_BACKTRACE=1 RUST_LOG=trace cargo run -p ip-api-api-demo --bin batch -- '8.8.8.8,4.4.4.4' 'YOUR_API_KEY'
*/

use std::{env, error};

use futures_lite::future::block_on;
use http_api_reqwest_client::{Client as _, ReqwestClient};
use ip_api_api::endpoints::batch::{Batch, BatchQuery};

fn main() -> Result<(), Box<dyn error::Error>> {
    pretty_env_logger::init();

    block_on(run())
}

async fn run() -> Result<(), Box<dyn error::Error>> {
    let ips = env::args()
        .nth(1)
        .unwrap()
        .split(',')
        .map(BatchQuery::new)
        .collect::<Vec<_>>();
    let key = env::args().nth(2).map(|x| x.into());

    let client = ReqwestClient::new()?;

    let batch = Batch::new(ips, key);

    let (res, rate_limit) = client.respond_endpoint(&batch).await?;

    println!("{:?}", res);
    println!("{:?}", rate_limit);

    Ok(())
}
