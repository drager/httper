//! HTTPer client
//!
//! In order to start making http(s) requests you need
//! to first initialize a `HttperClient` and then construct
//! the http request with the method you want to use.
//! For example for `DELETE` requests you call `.delete()`
//! on the `HttperClient` instance and then you call `.send()`
//! to send the request. See below for an example.
//!
//!
//! # Example
//!
//!```
//! extern crate httper;
//! extern crate tokio;
//! #[macro_use]
//! extern crate serde_derive;
//!
//! use httper::client::HttperClient;
//! use tokio::runtime::Runtime;
//!
//! fn main() {
//!    let mut rt = Runtime::new().unwrap();
//!
//!    let httper_client = HttperClient::new();
//!
//!    #[derive(Debug, Deserialize)]
//!    struct Contributor {
//!        id: u32,
//!        login: String,
//!    }
//!
//!    // Call .send() to fire the request and then call .json::<Vec<Contributor>>()
//!    // to turn the json response into a Vec containing Contributor.
//!    let result = rt.block_on(
//!        httper_client
//!            .get("https://api.github.com/repos/drager/httper/contributors")
//!            .send()
//!            .json::<Vec<Contributor>>(),
//!    );
//!
//!    println!("Contributors: {:?}", result);
//!}

use self::get::Get;
use self::payload_builder::PayloadBuilder;
use self::response_future::ResponseFuture;
use failure::Error;
use futures::future;
use http;
use hyper::{self, rt::Future};
use hyper_tls;
use native_tls;
use std::collections::HashMap;
use std::default::Default;
use std::error;

pub mod get;
pub mod payload_builder;
pub mod response_future;

type HttpClient<C> = hyper::Client<C, hyper::Body>;
type Url = str;

pub type HttpsClient = HttpClient<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>;

const PKG_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
const PKG_NAME: Option<&'static str> = option_env!("CARGO_PKG_NAME");

impl Default for HttperClient {
    fn default() -> Self {
        let http_client: hyper::Client<
            hyper_tls::HttpsConnector<hyper::client::HttpConnector>,
            _,
        > = build_https_client().expect("Failed to build HTTPs client");

        let mut headers = HashMap::new();

        let user_agent = hyper::header::USER_AGENT;

        headers.insert(
            user_agent.as_str().to_string(),
            format!(
                "{}/{}",
                PKG_NAME.unwrap_or("unknown_name"),
                PKG_VERSION.unwrap_or("unknown_version"),
            ),
        );

        HttperClient {
            http_client,
            headers,
        }
    }
}

pub type Headers = HashMap<String, String>;

#[derive(Debug, Clone)]
pub struct HttperClient {
    http_client: HttpsClient,
    headers: Headers,
}

impl HttperClient {
    /// Creates a new `HttperClient`.
    ///
    /// # Examples
    ///
    /// ```
    /// use httper::client::HttperClient;
    ///
    /// let httper_client = HttperClient::new();
    /// ```
    pub fn new() -> Self {
        HttperClient {
            ..HttperClient::default()
        }
    }

    /// Prepares a `GET` request to a given url `&str`.
    ///
    /// Call `.send()` to send the request.
    ///
    /// # Examples
    ///
    /// ```
    /// use httper::client::HttperClient;
    ///
    /// let httper_client = HttperClient::new();
    ///
    /// httper_client.get("https://testing.local").send();
    /// ```
    pub fn get(&self, url: &Url) -> Get {
        Get::new(self.request_builder(url, hyper::Method::GET), &self)
    }

    /// Prepares a `POST` request to a given url `&str`.
    ///
    /// Call `.send()` to send the request.
    ///
    /// # Examples
    /// ```
    /// use httper::client::HttperClient;
    ///
    /// let httper_client = HttperClient::new();
    ///
    /// httper_client.post("http://localhost:9090").payload("payload").send();
    /// ```
    pub fn post(&self, url: &Url) -> PayloadBuilder {
        PayloadBuilder::new(self.request_builder(url, hyper::Method::POST), &self)
    }

    /// Prepares a `DELETE` request to a given url `&str`.
    ///
    /// Call `.send()` to send the request.
    ///
    /// # Examples
    /// ```
    /// use httper::client::HttperClient;
    ///
    /// let httper_client = HttperClient::new();
    ///
    /// httper_client.delete("http://localhost:9090").send();
    /// ```
    pub fn delete(&self, url: &Url) -> PayloadBuilder {
        PayloadBuilder::new(self.request_builder(url, hyper::Method::DELETE), &self)
    }

    /// Prepares a `PUT` request to a given url `&str`.
    ///
    /// Call `.send()` to send the request.
    ///
    /// # Examples
    /// ```
    /// use httper::client::HttperClient;
    ///
    /// let httper_client = HttperClient::new();
    ///
    /// httper_client.put("http://localhost:9090").payload("payload").send();
    /// ```
    pub fn put(&self, url: &Url) -> PayloadBuilder {
        PayloadBuilder::new(self.request_builder(url, hyper::Method::PUT), &self)
    }

    /// Prepares a `PATCH` request to a given url `&str`.
    ///
    /// Call `.send()` to send the request.
    ///
    /// # Examples
    /// ```
    /// use httper::client::HttperClient;
    ///
    /// let httper_client = HttperClient::new();
    ///
    /// httper_client.patch("http://localhost:9090").payload("payload").send();
    /// ```
    pub fn patch(&self, url: &Url) -> PayloadBuilder {
        PayloadBuilder::new(self.request_builder(url, hyper::Method::PATCH), &self)
    }

    /// Get a `http::request::Builder` that will set the
    /// method and uri.
    ///
    /// # Errors
    /// Will return Err if the url couldn't be parsed into a `hyper::Uri`.
    fn request_builder(
        &self,
        url: &Url,
        method: hyper::Method,
    ) -> Result<http::request::Builder, Error> {
        let url = self.parse_url(url)?;
        let mut builder = http::request::Builder::new();
        builder.method(method).uri(url);
        Ok(builder)
    }

    /// Sends the request with the given `request_builder` and
    /// `payload`. Returns a `ResponseFuture`.
    fn send_request(
        &self,
        request_builder: Result<http::request::Builder, Error>,
        payload: hyper::Body,
        headers: &Headers,
    ) -> ResponseFuture {
        // Make key lowercase so when we merge our default headers with the new ones
        // it will replace the default ones if a new matches it even if the casing
        // isn't correct.
        let headers = headers
            .iter()
            .map(|(key, value)| (key.to_lowercase(), value.to_string()));

        // Create a new HashMap containing the passed in headers as well
        // as the default ones.
        let headers: Headers = self.headers.clone().into_iter().chain(headers).collect();

        let http_client = self.http_client.clone();

        ResponseFuture(Box::new(
            future::result(request_builder.and_then(|mut request_builder| {
                headers.iter().for_each(|(k, v)| {
                    request_builder.header(k.as_str(), v.as_str());
                });
                request_builder.body(payload).map_err(Error::from)
            })).and_then(move |request| http_client.request(request).map_err(Error::from)),
        ))
    }

    /// Parses the url `&str` to a `hyper::Uri`.
    ///
    /// # Errors
    /// Will return Err if the url couldn't be parsed into a `hyper::Uri`.
    fn parse_url(&self, url: &str) -> Result<hyper::Uri, Error> {
        url.parse::<hyper::Uri>().map_err(Error::from)
    }
}

/// Build a HTTPS client.
/// Returns a Result that contains the client on success.
fn build_https_client() -> Result<
    hyper::client::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>, hyper::Body>,
    Box<error::Error>,
> {
    let tls_connector = native_tls::TlsConnector::builder().build()?;

    let mut http_connector = hyper::client::HttpConnector::new(4);
    http_connector.enforce_http(false);

    let https_connector = hyper_tls::HttpsConnector::from((http_connector, tls_connector));

    let client = hyper::client::Client::builder().build(https_connector);

    Ok(client)
}
