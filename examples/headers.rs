extern crate httper;
extern crate tokio;

use httper::client::HttperClient;
use std::collections::HashMap;
use tokio::runtime::Runtime;

fn main() {
    let mut rt = Runtime::new().unwrap();

    let httper_client = HttperClient::new();

    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    // Call .send() to fire the request.
    let result = rt.block_on(
        httper_client
            .get("https://www.rust-lang.org/en-US/")
            .headers(headers)
            .send(),
    );

    println!("Result: {:?}", result);

    assert!(result.is_ok());
}
