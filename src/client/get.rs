use super::{Headers, HttperClient};
use client::response_future::ResponseFuture;
use failure::Error;
use http;
use hyper;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Get<'a> {
    pub request_builder: Result<http::request::Builder, Error>,
    pub client: &'a HttperClient,
    pub headers: Headers,
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
    /// httper_client.get("http://localhost:9090").headers(headers).send();
    /// ```
    pub fn headers(self, headers: Headers) -> Self {
        Get {
            headers,
            request_builder: self.request_builder,
            client: self.client,
        }
    }

    /// Sends the request and returns a `ResponseFuture`.
    pub fn send(self) -> ResponseFuture {
        self.client
            .send_request(self.request_builder, hyper::Body::empty(), &self.headers)
    }
}
