extern crate failure;
extern crate futures;
extern crate http;
extern crate hyper;
extern crate hyper_tls;
extern crate native_tls;
extern crate serde;
extern crate serde_json;
extern crate tokio;

#[cfg(test)]
#[macro_use]
extern crate serde_derive;

pub mod client;

#[cfg(test)]
mod tests {
    use super::client::{HttperClient, HttpsClient};
    use hyper::{self, rt::Future};
    use std::net::SocketAddr;
    use std::str;
    use std::thread;
    use tokio::runtime::current_thread::Runtime;

    fn start_server(body: &'static [u8], addr: &SocketAddr) {
        let new_svc = move || {
            hyper::service::service_fn_ok(move |_req| hyper::Response::new(hyper::Body::from(body)))
        };

        let server = hyper::server::Server::bind(addr)
            .serve(new_svc)
            .map_err(|e| eprintln!("server error: {}", e));

        hyper::rt::run(server);
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Data {
        name: String,
    }

    #[test]
    fn it_should_handle_get_requests() {
        use futures::Stream;

        let addr = ([127, 0, 0, 1], 9091).into();

        let mut rt = Runtime::new().unwrap();

        let buffer: &[u8] = br#"{"name": "Bumblebee"}"#;

        thread::spawn(move || {
            start_server(buffer, &addr);
        });

        let httper_client = HttperClient::<HttpsClient>::new();

        let result = rt.block_on(httper_client.get(&("http://".to_string() + &addr.to_string())));

        assert!(result.is_ok());
        let body = result.unwrap().into_body().concat2();
        assert_eq!(buffer, rt.block_on(body).unwrap().into_bytes());
    }

    #[test]
    fn it_should_be_able_to_be_chained_into_json_for_get() {
        let addr = ([127, 0, 0, 1], 9090).into();

        let mut rt = Runtime::new().unwrap();

        let buffer: &[u8] = br#"{"name": "Optimus Prime"}"#;

        thread::spawn(move || {
            start_server(buffer, &addr);
        });

        let httper_client = HttperClient::<HttpsClient>::new();

        let data = Data {
            name: "Optimus Prime".to_string(),
        };

        let result = rt.block_on(
            httper_client
                .get(&("http://".to_string() + &addr.to_string()))
                .json::<Data>(),
        );

        assert_eq!(data, result.unwrap());
    }

    #[test]
    fn it_should_handle_post_requests() {
        let addr = ([127, 0, 0, 1], 9092).into();

        let mut rt = Runtime::new().unwrap();

        let buffer: &[u8] = br#"{"name": "Bumblebee"}"#;

        thread::spawn(move || {
            start_server(buffer, &addr);
        });

        let httper_client = HttperClient::<HttpsClient>::new();

        let result = rt.block_on(httper_client.post(
            &("http://".to_string() + &addr.to_string()),
            hyper::Body::from(buffer),
        ));

        assert!(result.is_ok());
        assert_eq!(hyper::StatusCode::OK, result.unwrap().status());
    }

    #[test]
    fn it_should_be_able_to_be_chained_into_json_for_post() {
        let addr = ([127, 0, 0, 1], 9093).into();

        let mut rt = Runtime::new().unwrap();

        let buffer: &[u8] = br#"{"name": "Optimus Prime"}"#;

        thread::spawn(move || {
            start_server(buffer, &addr);
        });

        let httper_client = HttperClient::<HttpsClient>::new();

        let data = Data {
            name: "Optimus Prime".to_string(),
        };

        let result = rt.block_on(
            httper_client
                .post(
                    &("http://".to_string() + &addr.to_string()),
                    hyper::Body::from(buffer),
                )
                .json::<Data>(),
        );

        assert_eq!(data, result.unwrap());
    }

}
