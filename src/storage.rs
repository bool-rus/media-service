use futures_fs::FsPool;
use futures::Sink;
use futures::AsyncSink;
use futures::Async;
use bytes::*;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::marker::PhantomData;
use futures_fs::FsWriteSink;
use std::error::Error;


pub struct Buffer<I>(BytesMut, PhantomData<I>);

#[derive(Debug)]
pub struct BufferError;

impl Display for BufferError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str("превышен размер буфера")
    }
}
impl Error for BufferError {}

impl<I> Buffer<I> {
    pub fn new(capacity: usize) -> Self {
        Buffer(BytesMut::with_capacity(capacity),PhantomData)
    }
}

impl<I> AsRef<[u8]> for Buffer<I> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<I: AsRef<[u8]>> Sink for Buffer<I> {
    type SinkItem = I;
    type SinkError = BufferError;

    fn start_send(&mut self, item: Self::SinkItem) -> Result<AsyncSink<Self::SinkItem>, Self::SinkError> {
        let buf = &mut self.0;
        let item = item.as_ref();
        if buf.remaining_mut() < item.len() {
            Err(BufferError)
        } else {
            buf.put(item);
            Ok(AsyncSink::Ready)
        }
    }

    fn poll_complete(&mut self) -> Result<Async<()>, Self::SinkError> {
        Ok(Async::Ready(()))
    }

    fn close(&mut self) -> Result<Async<()>, Self::SinkError> {
        Ok(Async::Ready(()))
    }
}

#[derive(Debug)]
pub enum SharedError<E1: Error, E2: Error> {
    FIRST(E1),
    LAST(E2)
}

impl<E1: Error, E2:Error> Into<actix_web::Error> for SharedError<E1,E2> {
    fn into(self) -> actix_web::Error {
        actix_web::error::ErrorBadRequest("SharedError")
    }
}

pub struct SharedSink<S1,S2>{
    pub first: S1,
    pub last: S2,
    overhead: i8, //может быть 0, 1 и -1, 0 - все ок, 1 - вторым не принят чанкб -1 - первым не принят чанк
}


impl<S1,S2,E1, E2, I> Sink for SharedSink<S1,S2> where
    S1: Sink<SinkItem=I, SinkError=E1>,
    S2: Sink<SinkItem=I, SinkError=E2>,
    E1: Error,
    E2: Error,
    I: Clone
{
    type SinkItem = S1::SinkItem;
    type SinkError = SharedError<E1,E2>;

    fn start_send(&mut self, item: Self::SinkItem) -> Result<AsyncSink<Self::SinkItem>, Self::SinkError> {
        let (res1, res2) = match self.overhead {
            0 => (self.first.start_send(item.clone()), self.last.start_send(item)),
            _x if _x>0 => {
                self.overhead = 0;
                (Ok(AsyncSink::Ready), self.last.start_send(item),)
            }
            _x if _x<0 => {
                self.overhead = 0;
                (self.first.start_send(item), Ok(AsyncSink::Ready),)
            },
            _ => unreachable!()
        };
        match (res1, res2) {
            (Ok(AsyncSink::Ready),Ok(AsyncSink::Ready)) => Ok(AsyncSink::Ready),
            (Ok(AsyncSink::Ready), Ok(AsyncSink::NotReady(item))) => {
                self.overhead = 1;
                Ok(AsyncSink::NotReady(item))
            },
            (Ok(AsyncSink::NotReady(item)),Ok(AsyncSink::Ready)) => {
                self.overhead = -1;
                Ok(AsyncSink::NotReady((item)))
            }
            (Ok(AsyncSink::NotReady(item)),Ok(AsyncSink::NotReady(_))) => Ok(AsyncSink::NotReady(item)),
            (Err(e), _) => {
                let e = e;
                Err(SharedError::FIRST(e))
            },
            (_, Err(e)) => Err(SharedError::LAST(e)),
        }
    }

    fn poll_complete(&mut self) -> Result<Async<()>, Self::SinkError> {
        match (self.first.poll_complete(), self.last.poll_complete()) {
            (Ok(Async::Ready(_)),Ok(Async::Ready(_))) => Ok(Async::Ready(())),
            (Err(e), _) => Err(SharedError::FIRST(e)),
            (_, Err(e)) => Err(SharedError::LAST(e)),
            _ => Ok(Async::NotReady)
        }
    }

    fn close(&mut self) -> Result<Async<()>, Self::SinkError> {
        match (self.first.close(), self.last.close()) {
            (Ok(Async::Ready(_)),Ok(Async::Ready(_))) => Ok(Async::Ready(())),
            (Err(e), _) => Err(SharedError::FIRST(e)),
            (_, Err(e)) => Err(SharedError::LAST(e)),
            _ => Ok(Async::NotReady)
        }
    }
}

pub trait Sharing<T: Sink> {
    fn share(self, another: T) -> SharedSink<Self, T> where Self: Sized + Sink {
        SharedSink {
            first: self,
            last: another,
            overhead: 0
        }
    }
}

impl<S1: Sink, S2: Sink> Sharing<S2> for S1 {}

pub fn make_writer(name: String) -> FsWriteSink {
    FsPool::default().write(name, Default::default())
}