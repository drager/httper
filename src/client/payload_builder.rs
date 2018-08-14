use super::{Headers, HttperClient};
use client::response_future::ResponseFuture;
use failure::Error;
use http;
use hyper;
use std::collections::HashMap;

#[derive(Debug)]
pub struct PayloadBuilder<'a> {
    pub request_builder: Result<http::request::Builder, Error>,
    pub client: &'a HttperClient,
    pub payload: Option<hyper::Body>,
    pub headers: Headers,
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
            headers: HashMap::new(),
        }
    }

    /// Attach headers to the request
    ///
    /// # Examples
    /// ```
    /// use httper::client::HttperClient;
    /// use std::collections::HashMap;
    ///
    /// let httper_client = HttperClient::new();
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("Content-Type".to_string(), "application/json".to_string());
    ///
    /// httper_client
    ///     .post("http://localhost:9090")
    ///     .headers(headers)
    ///     .payload("payload")
    ///     .send();
    /// ```
    pub fn headers(self, headers: Headers) -> Self {
        PayloadBuilder {
            headers,
            request_builder: self.request_builder,
            client: self.client,
            payload: self.payload,
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
            headers: self.headers,
        }
    }

    /// Sends the request and returns a `ResponseFuture`.
    ///
    pub fn send(self) -> ResponseFuture {
        self.client.send_request(
            self.request_builder,
            self.payload.unwrap_or_else(|| hyper::Body::empty()),
            &self.headers,
        )
    }
}
