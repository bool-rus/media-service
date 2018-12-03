use futures_fs::{FsPool};
use futures::{Future, Stream};
use futures::future;
use actix_web::HttpRequest;
use bytes::*;
use actix_web::error;

pub fn invoke_file(_req: &HttpRequest) -> impl Future<Item=Bytes, Error=actix_web::Error> {

    let initial_size = 128*1024usize;
    FsPool::default().read("/Users/bool/Downloads/alice.torrent",Default::default())
        .map_err(error::ErrorInternalServerError)
        .fold(BytesMut::with_capacity(initial_size),|mut buf,chunk| {
            buf.put(chunk);
            future::ok::<BytesMut, actix_web::Error>(buf)
        })
        .map(|buf|buf.into())
}