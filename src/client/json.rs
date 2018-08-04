use futures::{Async, Future, Poll};
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct Json<F, T> {
    pub future: F,
    pub _t: PhantomData<T>,
}

impl<F, T> Future for Json<F, T>
where
    F: Future,
{
    type Item = T;
    type Error = F::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        unimplemented!("do your thing here")
    }
}
