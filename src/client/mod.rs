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

        headers.insert(
            hyper::header::USER_AGENT,
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

#[derive(Debug, Clone)]
pub struct HttperClient {
    http_client: HttpsClient,
    headers: HashMap<hyper::header::HeaderName, String>,
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
    pub fn new() -> HttperClient {
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
        let mut builder = self.request_with_default_headers();
        builder.method(method).uri(url);
        Ok(builder)
    }

    /// Sends the request with the given `request_builder` and
    /// `payload`. Returns a `ResponseFuture`.
    fn send_request(
        &self,
        request_builder: Result<http::request::Builder, Error>,
        payload: hyper::Body,
    ) -> ResponseFuture {
        let http_client = self.http_client.clone();
        ResponseFuture(Box::new(
            future::result(request_builder.and_then(|mut request_builder| {
                request_builder.body(payload).map_err(Error::from)
            })).and_then(move |request| http_client.request(request).map_err(Error::from)),
        ))
    }

    /// Setup a `http::request::Builder` with default headers,
    /// `self.headers`.
    fn request_with_default_headers(&self) -> http::request::Builder {
        let mut request = hyper::Request::builder();

        self.headers.iter().for_each(|(k, v)| {
            request.header(k.as_str(), v.as_str());
        });

        request
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
