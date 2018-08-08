use super::HttperClient;
use client::response_future::ResponseFuture;
use failure::Error;
use http;
use hyper;

#[derive(Debug)]
pub struct Get<'a> {
    pub request_builder: Result<http::request::Builder, Error>,
    pub client: &'a HttperClient,
}

impl<'a> Get<'a> {
    /// Creates a new `Get`.
    pub fn new(
        request_builder: Result<http::request::Builder, Error>,
        client: &'a HttperClient,
    ) -> Self {
        Get {
            request_builder,
            client,
        }
    }

    /// Sends the request and returns a `ResponseFuture`.
    pub fn send(self) -> ResponseFuture {
        self.client
            .send_request(self.request_builder, hyper::Body::empty())
    }
}
