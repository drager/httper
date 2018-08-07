extern crate httper;
extern crate hyper;
extern crate tokio;

use httper::client::HttperClient;
use std::net::SocketAddr;
use std::thread;
use tokio::prelude::future::Future;
use tokio::runtime::Runtime;

/// Spins up a server using hyper.
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
    let addr = ([127, 0, 0, 1], 9092).into();

    let mut rt = Runtime::new().unwrap();

    let buffer: &[u8] = br#"{"name": "Bumblebee"}"#;

    // Spin up a temporary server so we can
    // post some data somewhere.
    thread::spawn(move || {
        start_server(buffer, &addr);
    });

    let httper_client: HttperClient = HttperClient::new();

    let result =
        rt.block_on(httper_client.post(&("http://".to_string() + &addr.to_string()), buffer));

    println!("Result: {:?}", result);

    assert!(result.is_ok());
}
