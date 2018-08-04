extern crate httper;
extern crate hyper;
extern crate tokio;
#[macro_use]
extern crate serde_derive;

use httper::client::{HttperClient, HttpsClient};
use hyper::rt::Future;
use std::net::SocketAddr;
use std::thread;
use tokio::runtime::Runtime;

fn start_server(body: &'static [u8], addr: &SocketAddr) {
    let new_svc = move || {
        hyper::service::service_fn_ok(move |_req| hyper::Response::new(hyper::Body::from(body)))
    };

    let server = hyper::server::Server::bind(addr)
        .serve(new_svc)
        .map_err(|e| eprintln!("server error: {}", e));

    hyper::rt::run(server);
}

fn main() {
    let mut rt = Runtime::new().unwrap();

    let addr: SocketAddr = ([127, 0, 0, 1], 9090).into();
    let buffer: &[u8] = br#"{"name": "Bumblebee"}"#;

    thread::spawn(move || {
        start_server(buffer, &addr);
    });

    let httper_client = HttperClient::<HttpsClient>::new();

    #[derive(Debug, Deserialize, PartialEq)]
    struct Data {
        name: String,
    }

    let url = format!("http://{}", addr);
    let result = rt.block_on(httper_client.get(&url).json::<Data>());

    assert!(result.is_ok());
}
