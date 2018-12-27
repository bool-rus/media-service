use futures::Stream;
use bytes::Bytes;
use futures::Async;

pub type HashString = [u8; 20];

#[derive(Debug,Fail)]
#[fail(display="{}",0)]
pub struct TorrentError(pub String);

pub trait TorrentClient {
    fn download(&mut self) -> SizedStream;
    fn download_file(&mut self, num: usize) -> SizedStream;
}

pub struct SizedStream {
    size: usize,
    stream: Box<Stream<Item=Bytes,Error=failure::Error>>
}

impl SizedStream {
    pub fn new<S>(size: usize, stream: S) -> Self
        where S: Stream<Item=Bytes,Error=failure::Error> + 'static {
        SizedStream {
            size,
            stream: Box::new(stream)
        }
    }
    pub fn size(&self) -> usize {
        self.size
    }
}

impl Stream for SizedStream {
    type Item = Bytes;
    type Error = failure::Error;

    fn poll(&mut self) -> Result<Async<Option<<Self as Stream>::Item>>, <Self as Stream>::Error> {
        self.stream.poll()
    }
}