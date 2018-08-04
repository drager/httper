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
