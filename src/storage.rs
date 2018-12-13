use futures_fs::FsPool;
use futures::Sink;
use futures::AsyncSink;
use futures::Async;
use bytes::*;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::marker::PhantomData;
use futures_fs::FsWriteSink;
use std::error::Error;

pub struct CachedSink<T: Sink> {
    cache: BytesMut,
    sink: T,
    synchronized: bool
}

impl<T: Sink> CachedSink<T> {
    pub fn new(sink: T, size: usize) -> CachedSink<T> {
        CachedSink {
            cache: BytesMut::with_capacity(size),
            sink,
            synchronized: true
        }
    }
}

impl<T: Sink> AsRef<[u8]> for CachedSink<T> {
    fn as_ref(&self) -> &[u8] {
        self.cache.as_ref()
    }
}

#[derive(Debug, Fail)]
#[fail(display = "Cache buffer exceeded")]
pub struct CachedSinkError;


impl<T,I,E> Sink for CachedSink<T>
    where T: Sink<SinkItem=I, SinkError=E>,
    I: AsRef<[u8]>,
    E: Error + Sync + Send + 'static
{
    type SinkItem = I;
    type SinkError = failure::Error;

    fn start_send(&mut self, item: Self::SinkItem) -> Result<AsyncSink<Self::SinkItem>, Self::SinkError> {
        if self.synchronized {
            let item = item.as_ref();
            if item.len() > self.cache.remaining_mut() {
                return Err(CachedSinkError.into())
            }
            self.cache.put(item);
        }
        let ret = self.sink.start_send(item);
        match &ret {
            Ok(AsyncSink::NotReady(_)) => self.synchronized = false,
            Ok(AsyncSink::Ready) => self.synchronized = true,
            _ => {},
        }
        ret.map_err(From::from)
    }

    fn poll_complete(&mut self) -> Result<Async<()>, Self::SinkError> {
        self.sink.poll_complete().map_err(From::from)
    }

    fn close(&mut self) -> Result<Async<()>, Self::SinkError> {
        self.sink.close().map_err(From::from)
    }
}

pub fn make_writer(name: String) -> FsWriteSink {
    FsPool::default().write(name, Default::default())
}

#[cfg(test)]
mod test {
    use storage::CachedSinkError;

    #[test]
    fn from_cached_to_atix() {
        actix_web::Error::from(Into::<failure::Error>::into(CachedSinkError));
    }
}