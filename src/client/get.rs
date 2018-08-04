use futures::Async;
use futures::Future;
use futures::Poll;

#[derive(Debug)]
pub struct Get<H, F>
where
    F: Future,
{
    pub httper: H,
    pub future: F,
}

/*impl<U, A> Future for Get<A>*/
//where
//A: Future,
//{
//type Item = U;
//type Error = A::Error;

//fn poll(&mut self) -> Poll<U, A::Error> {
//let e = match self.future.poll() {
//Ok(Async::NotReady) => return Ok(Async::NotReady),
//Ok(Async::Ready(e)) => Ok(e),
//Err(e) => Err(e),
//};
//e.map(self.f.take().expect("cannot poll Get twice"))
//.map(Async::Ready)
//}
/*}*/
