use self::response_future::ResponseFuture;
use failure::Error;
use futures::future;
use hyper::{
    self, rt::Future,
};
use hyper_tls;
use native_tls;
use std::default::Default;
use std::error;

pub mod response_future;

type HttpClient<C> = hyper::Client<C, hyper::Body>;
type Url = str;

pub type HttpsClient = HttpClient<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>;

#[derive(Debug)]
pub struct HttperClientBuilder<C> {
    http_client: C,
}

impl Default for HttperClient<HttpsClient> {
    fn default() -> Self {
        let http_client: hyper::Client<
            hyper_tls::HttpsConnector<hyper::client::HttpConnector>,
            _,
        > = build_https_client().expect("Failed to build HTTPs client");

        HttperClient { http_client }
    }
}

impl<C> HttperClientBuilder<C> {
    pub fn new(c: C) -> Self {
        HttperClientBuilder { http_client: c }
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
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttperClient<C> {
    http_client: C,
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
    pub fn get<'a>(self, url: &Url) -> ResponseFuture<'a> {
        ResponseFuture(Box::new(
            future::result(self.parse_url(url))
                .and_then(move |url| self.http_client.get(url).map_err(Error::from)),
        ))
    }

    pub fn post<'a>(
        &'a self,
        url: &Url,
        payload: hyper::Body,
    ) -> impl Future<Item = hyper::Response<hyper::Body>, Error = Error> + Sized + 'a {
        future::result(
            self.parse_url(url)
                .and_then(|url| hyper::Request::post(url).body(payload).map_err(Error::from)),
        ).and_then(move |request| self.http_client.request(request).map_err(Error::from))
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
