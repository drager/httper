extern crate httper;
extern crate tokio;

use httper::client::HttperClient;
use tokio::runtime::Runtime;

fn main() {
    let mut rt = Runtime::new().unwrap();

    let httper_client = HttperClient::new();

    let result = rt.block_on(httper_client.get("https://www.rust-lang.org/en-US/"));

    println!("Result: {:?}", result);

    assert!(result.is_ok());
}
