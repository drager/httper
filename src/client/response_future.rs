use failure::Error;
use futures::{future, Async, Future, Poll, Stream};
use hyper;
use serde::de::DeserializeOwned;
use serde_json;
use std::fmt;

pub struct ResponseFuture<'a>(
    pub Box<Future<Item = hyper::Response<hyper::Body>, Error = Error> + 'a + Send>,
);

impl<'a> Future for ResponseFuture<'a> {
    type Item = hyper::Response<hyper::Body>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let e = match self.0.poll() {
            Ok(Async::NotReady) => return Ok(Async::NotReady),
            Ok(Async::Ready(e)) => Ok(e),
            Err(e) => Err(e),
        };
        e.map(Async::Ready)
    }
}

impl<'a> ResponseFuture<'a> {
    /// Deserialize the response json body into a `T`.
    /// Returns a Future containing the deserialized body.
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
    ///     httper_client.get("https://testing.local").json::<Data>();
    /// }
    /// ```
    ///
    pub fn json<T: 'a>(self) -> impl Future<Item = T, Error = Error> + Sized + 'a
    where
        T: DeserializeOwned + fmt::Debug,
    {
        self.0.and_then(|response| {
            response
                .into_body()
                .map_err(Error::from)
                .concat2()
                .and_then(|body| {
                    future::result(serde_json::from_slice::<T>(&body).map_err(Error::from))
                })
        })
    }
}
