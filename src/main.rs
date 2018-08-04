extern crate httper;
extern crate tokio;

use httper::client::{Httper, HttperClient, HttpsClient};
use tokio::runtime::Runtime;

fn main() {
    let mut rt = Runtime::new().unwrap();

    let httper_client = HttperClient::<HttpsClient>::new();

    #[derive(Debug, Deserialize, PartialEq)]
    struct Data {
        name: String,
    }

    let result = httper_client.get("http://localhost:8000").json::<Data>();
    //println!("Result {:?}", result);
    let a = rt.block_on(result);
}
