extern crate httper;
extern crate tokio;
#[macro_use]
extern crate serde_derive;

use httper::client::HttperClient;
use tokio::runtime::Runtime;

fn main() {
    let mut rt = Runtime::new().unwrap();

    let httper_client = HttperClient::new();

    #[derive(Debug, Deserialize, PartialEq)]
    struct Contributor {
        id: u32,
        login: String,
    }

    // Call .send() to fire the request and then call .json::<Vec<Contributor>>()
    // to turn the json response into a Vec containing Contributor.
    let result = rt.block_on(
        httper_client
            .get("https://api.github.com/repos/drager/httper/contributors")
            .send()
            .json::<Vec<Contributor>>(),
    );

    println!("Contributors: {:?}", result);

    assert!(result.is_ok());
}
