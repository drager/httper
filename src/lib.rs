extern crate failure;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate native_tls;
extern crate serde;
extern crate serde_json;
extern crate tokio;

#[cfg(test)]
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
extern crate tokio_mockstream;

pub mod client;

#[cfg(test)]
mod tests {
    use super::client::{HttperClient, HttpsClient};
    use futures::{future, Future};
    use hyper;
    use std::io;
    use std::io::Read;
    use std::io::Write;
    use std::str;
    use std::thread;
    use tokio::runtime::current_thread::Runtime;
    use tokio_mockstream::MockStream;
    use std::net::SocketAddr;

    fn start_server(body: &'static [u8], addr: &SocketAddr) {
        let new_svc = move || {
            hyper::service::service_fn_ok(move |_req| hyper::Response::new(hyper::Body::from(body)))
        };

        let server = hyper::server::Server::bind(addr)
            .serve(new_svc)
            .map_err(|e| eprintln!("server error: {}", e));

        hyper::rt::run(server);
    }

    #[derive(Debug)]
    struct MockConnector<'a> {
        buffer: &'a [u8],
    }

    impl<'a> hyper::client::connect::Connect for MockConnector<'a> {
        type Transport = MockStream;
        type Error = io::Error;
        type Future = Box<
            Future<Item = (MockStream, hyper::client::connect::Connected), Error = io::Error>
                + Send,
        >;

        fn connect(&self, dst: hyper::client::connect::Destination) -> Self::Future {
            println!("Destination: {:?}", dst);
            let mut mock_stream = MockStream::empty();
            mock_stream
                .write(b"HTTP/1.1 200 OK\r\nContent-Length: 512\r\n\r\n")
                .unwrap();
            mock_stream.write(self.buffer).unwrap();
            Box::new(future::ok((
                mock_stream,
                hyper::client::connect::Connected::new(),
            )))
        }
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Data {
        name: String,
    }

    #[test]
    fn it_should_return_json() {
        let addr = ([127, 0, 0, 1], 9090).into();

        let mut rt = Runtime::new().unwrap();

        let buffer: &[u8] = br#"{"name": "Optimus Prime"}"#;

        thread::spawn(move || {
            start_server(buffer, &addr);
        });
        //let buffer: &[u8] = response_string.as_bytes();
        //let connector = MockConnector { buffer };

        //let hyper_client = hyper::client::Client::builder().build::<_, hyper::Body>(connector);

        //let client: HttperClient<hyper::client::Client<MockConnector, hyper::Body>> = HttperClientBuilder::new(hyper_client).build();

        let httper_client = HttperClient::<HttpsClient>::new();

        let data = Data {
            name: "Optimus Prime".to_string(),
        };

        let result = rt.block_on(httper_client.get_json::<Data>(&("http://".to_string() + &addr.to_string())));

        assert_eq!(data, result.unwrap());
    }
}
