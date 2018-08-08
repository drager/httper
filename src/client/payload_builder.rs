use super::HttperClient;
use client::response_future::ResponseFuture;
use failure::Error;
use http;
use hyper;

#[derive(Debug)]
pub struct PayloadBuilder<'a> {
    pub request_builder: Result<http::request::Builder, Error>,
    pub client: &'a HttperClient,
    pub payload: Option<hyper::Body>,
}

impl<'a> PayloadBuilder<'a> {
    /// Creates a new `PayloadBuilder`.
    ///
    pub fn new(
        request_builder: Result<http::request::Builder, Error>,
        client: &'a HttperClient,
    ) -> Self {
        PayloadBuilder {
            request_builder,
            client,
            payload: None,
        }
    }

    /// Attaches payload to the request.
    ///
    pub fn payload<P: Into<hyper::Body> + Send>(self, payload: P) -> PayloadBuilder<'a>
    where
        hyper::Body: From<P>,
    {
        PayloadBuilder {
            request_builder: self.request_builder,
            client: self.client,
            payload: Some(hyper::Body::from(payload)),
        }
    }

    /// Sends the request and returns a `ResponseFuture`.
    ///
    pub fn send(self) -> ResponseFuture {
        self.client.send_request(
            self.request_builder,
            self.payload.unwrap_or_else(|| hyper::Body::empty()),
        )
    }
}
