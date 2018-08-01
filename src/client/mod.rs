use failure::Error;
use futures::future;
use hyper::{
    self, rt::{Future, Stream},
};
use hyper_tls;
use native_tls;
use serde::de::DeserializeOwned;
use serde_json;
use std::default::Default;
use std::error;
use std::fmt;

type HttpClient<C> = hyper::Client<C, hyper::Body>;

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

    /// Perform a get request to a given url `&str`
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
    pub fn get<'a>(
        &'a self,
        url: &str,
    ) -> impl Future<Item = hyper::Response<hyper::Body>, Error = Error> + Sized + 'a {
        future::result(self.parse_url(url))
            .and_then(move |url| self.http_client.get(url).map_err(Error::from))
    }

    /// Perform a get request to a given url `&str` and deserialzie
    /// the response json body into a `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate httper;
    ///
    /// #[macro_use]
    /// extern crate serde_derive;
    ///
    /// use httper::client::{HttperClient, HttpsClient};
    ///
    /// fn main() {
    ///
    ///     #[derive(Debug, Deserialize)]
    ///     struct Data {
    ///         name: String,
    ///     }
    ///
    ///     let httper_client = HttperClient::<HttpsClient>::new();
    ///
    ///     let data = Data {
    ///         name: "Optimus Prime".to_string(),
    ///     };
    ///
    ///     httper_client.get_json::<Data>("https://testing.local");
    /// }
    /// ```
    ///
    pub fn get_json<'a, T: 'a>(
        &'a self,
        url: &str,
    ) -> impl Future<Item = T, Error = Error> + Sized + 'a
    where
        T: DeserializeOwned + fmt::Debug,
    {
        self.get(url).and_then(move |response| {
            response
                .into_body()
                .map_err(Error::from)
                .concat2()
                .and_then(move |body| self.deserialize_data::<T>(&body))
        })
    }

    /// Parses the url `&str` to a `hyper::Uri`.
    ///
    /// # Errors
    /// Will return Err if the url couldn't be parsed into a `hyper::Uri`.
    ///
    fn parse_url(&self, url: &str) -> Result<hyper::Uri, Error> {
        url.parse::<hyper::Uri>().map_err(Error::from)
    }

    /// Deserializes the body `hyper::Chunk` to a `T`.
    ///
    /// # Errors
    /// Will return Err if the body couldn't be deserialzied into a `T`.
    ///
    fn deserialize_data<T>(
        &self,
        body: &hyper::Chunk,
    ) -> impl Future<Item = T, Error = Error> + Sized
    where
        T: DeserializeOwned + fmt::Debug,
    {
        future::result(serde_json::from_slice::<T>(&body).map_err(Error::from))
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
