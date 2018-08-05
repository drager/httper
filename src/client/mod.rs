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

pub mod response_future;

type HttpClient<C> = hyper::Client<C, hyper::Body>;
type Url = str;

pub type HttpsClient = HttpClient<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>;

const PKG_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
const PKG_NAME: Option<&'static str> = option_env!("CARGO_PKG_NAME");

#[derive(Debug)]
pub struct HttperClientBuilder<C> {
    http_client: C,
    headers: HashMap<hyper::header::HeaderName, String>,
}

impl Default for HttperClient<HttpsClient> {
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

impl<C> HttperClientBuilder<C> {
    pub fn new(c: C) -> Self {
        HttperClientBuilder {
            http_client: c,
            headers: HashMap::new(),
        }
    }

    pub fn http_client(self, http_client: C) -> Self {
        HttperClientBuilder {
            http_client,
            ..self
        }
    }

    pub fn build(self) -> HttperClient<C> {
        HttperClient {
            http_client: self.http_client,
            headers: self.headers,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttperClient<C> {
    http_client: C,
    headers: HashMap<hyper::header::HeaderName, String>,
}

impl<C> HttperClient<HttpClient<C>>
where
    C: hyper::client::connect::Connect + 'static,
{
    /// Creates a new `HttperClient`.
    ///
    /// # Examples
    ///
    /// ```
    /// use httper::client::{HttperClient, HttpsClient};
    ///
    /// let httper_client = HttperClient::<HttpsClient>::new();
    /// ```
    ///
    pub fn new() -> HttperClient<HttpsClient> {
        HttperClient {
            ..HttperClient::default()
        }
    }

    /// Performs a get request to a given url `&str`
    ///
    /// # Examples
    ///
    /// ```
    /// use httper::client::{HttperClient, HttpsClient};
    ///
    /// let httper_client = HttperClient::<HttpsClient>::new();
    ///
    /// httper_client.get("https://testing.local");
    /// ```
    ///
    pub fn get(&self, url: &Url) -> ResponseFuture {
        let mut request = self.request_with_default_headers();
        let http_client = self.http_client.clone();

        ResponseFuture(Box::new(
            future::result(self.parse_url(url).and_then(|url| {
                request
                    .method(hyper::Method::GET)
                    .uri(url)
                    .body(hyper::Body::empty())
                    .map_err(Error::from)
            })).and_then(move |request| http_client.request(request).map_err(Error::from)),
        ))
    }

    /// Performs a post request to a given url `&str` with
    /// the provided payload `hyper::Body`.
    ///
    /// # Examples
    /// ```
    /// extern crate httper;
    /// extern crate hyper;
    ///
    /// use httper::client::{HttperClient, HttpsClient};
    ///
    /// fn main() {
    ///     let httper_client = HttperClient::<HttpsClient>::new();
    ///
    ///     httper_client.post("http://localhost:9090", hyper::Body::from("payload"));
    /// }
    /// ```
    ///
    pub fn post<P: Into<hyper::Body> + Send>(&self, url: &Url, payload: P) -> ResponseFuture
    where
        hyper::Body: From<P>,
    {
        let mut request = self.request_with_default_headers();
        let http_client = self.http_client.clone();

        ResponseFuture(Box::new(
            future::result(self.parse_url(url).and_then(|url| {
                request
                    .method(hyper::Method::POST)
                    .uri(url)
                    .body(hyper::Body::from(payload))
                    .map_err(Error::from)
            })).and_then(move |request| http_client.request(request).map_err(Error::from)),
        ))
    }

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
    ///
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
