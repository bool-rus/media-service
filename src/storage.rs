use futures_fs::*;
use bytes::*;
use std::marker::PhantomData;
use failure::*;
use tokio::prelude::*;



struct NullSink<I,E>(PhantomData<I>,PhantomData<E>);
impl<I,E> NullSink<I,E> {
    fn new() -> Self {
        NullSink(PhantomData,PhantomData)
    }
}

impl<I,E> Sink for NullSink<I,E> {
    type SinkItem = I;
    type SinkError = E;

    fn start_send(&mut self, _: <Self as Sink>::SinkItem) -> Result<AsyncSink<<Self as Sink>::SinkItem>, <Self as Sink>::SinkError> {
        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Result<Async<()>, <Self as Sink>::SinkError> {
        Ok(Async::Ready(()))
    }

    fn close(&mut self) -> Result<Async<()>, <Self as Sink>::SinkError> {
        Ok(Async::Ready(()))
    }
}

pub struct CachedSink<T: Sink> {
    cache: BytesMut,
    sink: T,
    synchronized: bool,
}

impl<T: Sink> CachedSink<T> {
    pub fn new(sink: T, size: usize) -> CachedSink<T> {
        CachedSink {
            cache: BytesMut::with_capacity(size),
            sink,
            synchronized: true,
        }
    }
    fn to_bytes(self) -> Bytes {
        self.cache.into()
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


impl<T, I, E> Sink for CachedSink<T>
    where T: Sink<SinkItem=I, SinkError=E>,
          I: AsRef<[u8]>,
          E: Into<Error> + Sync + Send + 'static
{
    type SinkItem = I;
    type SinkError = Error;

    fn start_send(&mut self, item: Self::SinkItem) -> Result<AsyncSink<Self::SinkItem>, Self::SinkError> {
        if self.synchronized {
            let item = item.as_ref();
            if item.len() > self.cache.remaining_mut() {
                return Err(CachedSinkError.into());
            }
            self.cache.put(item);
        }
        let ret = self.sink.start_send(item);
        match &ret {
            Ok(AsyncSink::NotReady(_)) => self.synchronized = false,
            Ok(AsyncSink::Ready) => self.synchronized = true,
            _ => {}
        }
        ret.map_err(Into::into)
    }

    fn poll_complete(&mut self) -> Result<Async<()>, Self::SinkError> {
        self.sink.poll_complete().map_err(Into::into)
    }

    fn close(&mut self) -> Result<Async<()>, Self::SinkError> {
        self.sink.close().map_err(Into::into)
    }
}

pub fn make_writer(name: String) -> FsWriteSink {
    FsPool::default().write(name, Default::default())
}

pub fn make_reader(hash: String) -> FsReadStream {
    FsPool::default().read(hash, Default::default())
}

pub fn read(hash: String) -> impl Future<Item=Bytes, Error=Error> {
    let size = std::fs::metadata(&hash).unwrap().len() as usize;
    make_reader(hash)
        .from_err::<Error>()
        .forward(CachedSink::new(NullSink::<_,Error>::new(),size))
        .map(|(_,sink)|sink.to_bytes())
}

#[cfg(test)]
mod test {
    use storage::CachedSinkError;
    use std::io;
    use failure::Fail;
    use failure::Error;

    #[test]
    fn from_cached_to_atix() {
        actix_web::Error::from(Into::<failure::Error>::into(CachedSinkError));
    }
    fn make_failure_err(err: io::Error) -> failure::Error {
        err.into()
    }
    #[test]
    fn from_io_to_failure() {
        let err1 = io::Error::from_raw_os_error(0);
        //let err2 = io::Error::from_raw_os_error(0);
        let err1 = failure::Error::from(err1);
        //let err2 = make_failure_err(err2);
    }
}