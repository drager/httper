extern crate httper;
extern crate tokio;

#[macro_use]
extern crate serde_derive;

use httper::client::{HttperClient, HttpsClient};
use tokio::runtime::Runtime;

fn main() {
    let mut rt = Runtime::new().unwrap();

    let httper_client = HttperClient::<HttpsClient>::new();

    #[derive(Debug, Deserialize, PartialEq)]
    struct Data {
        name: String,
    }

    let result = rt.block_on(httper_client.get(&("http://localhost:6000")).json::<Data>());
}
