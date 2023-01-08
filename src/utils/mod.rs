use std::{task::Poll};
use futures::{Stream, Future, StreamExt, FutureExt, TryStream, TryStreamExt, TryFuture, TryFutureExt};
use tokio::fs::{ReadDir, DirEntry};

#[derive(Debug)]
#[repr(transparent)]
pub struct ReadDirStream {
    inner: ReadDir
}

impl ReadDirStream {
    #[inline]
    pub fn new (inner: ReadDir) -> Self {
        return Self { inner }
    }

    #[inline]
    pub fn into_inner (self) -> ReadDir {
        return self.inner
    }
}

impl Stream for ReadDirStream {
    type Item = std::io::Result<DirEntry>;

    #[inline]
    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        self.inner.poll_next_entry(cx).map(std::io::Result::transpose)
    }
}

pin_project_lite::pin_project! {
    pub struct FlattenOkIter<St: TryStream> where St::Ok: IntoIterator {
        #[pin]
        stream: St,
        iter: Option<<St::Ok as IntoIterator>::IntoIter>
    }
}

impl<St: TryStream> FlattenOkIter<St> where St::Ok: IntoIterator {
    #[inline]
    pub fn new (stream: St) -> Self {
        return Self {
            stream,
            iter: None
        }
    }
}

impl<St: TryStream> Stream for FlattenOkIter<St> where St::Ok: IntoIterator {
    type Item = Result<<St::Ok as IntoIterator>::Item, St::Error>;

    #[inline]
    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        let poll = match this.stream.try_poll_next(cx) {
            Poll::Ready(Some(Ok(iter))) => {
                let mut iter = iter.into_iter();
                if let Some(value) = iter.next() {
                    *this.iter = Some(iter);
                    return Poll::Ready(Some(Ok(value)))
                }
                Poll::Pending
            },
            Poll::Ready(Some(Err(e))) => return Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending
        };

        if let Some(iter) = this.iter {
            if let Some(value) = iter.next() {
                return Poll::Ready(Some(Ok(value)))
            }
        }

        return poll
    }
}

#[inline]
pub fn reduce<St, F, Fut> (st: St, mut f: F) -> impl Future<Output = Option<St::Item>> where
    St: Stream + Sized,
    F: FnMut(St::Item, St::Item) -> Fut,
    Fut: Future<Output = St::Item>
{
    return st.fold(None, move |lhs: Option<St::Item>, rhs: St::Item| {
        match lhs {
            Some(lhs) => futures::future::Either::Left(f(lhs, rhs).map(Some)),
            None => futures::future::Either::Right(futures::future::ready(Some(rhs)))
        }
    })
}

#[inline]
pub fn try_reduce<St, F, Fut> (st: St, mut f: F) -> impl TryFuture<Ok = Option<St::Ok>, Error = St::Error> where
    St: TryStream + Sized,
    F: FnMut(St::Ok, St::Ok) -> Fut,
    Fut: TryFuture<Ok = St::Ok, Error = St::Error>
{
    return st.try_fold(None, move |lhs: Option<St::Ok>, rhs: St::Ok| {
        match lhs {
            Some(lhs) => futures::future::Either::Left(f(lhs, rhs).map_ok(Some)),
            None => futures::future::Either::Right(futures::future::ready(Ok(Some(rhs))))
        }
    })
}

#[inline]
pub fn stream_and_then<St, F, T, E, U> (st: St, mut f: F) -> impl Stream<Item = Result<U, E>> where
    St: Stream<Item = Result<T, E>>,
    F: FnMut(T) -> Result<U, E>
{
    return st.map(move |x| match x {
        Ok(t) => f(t),
        Err(e) => Err(e),
    })
}