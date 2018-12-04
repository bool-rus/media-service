extern crate http;

use futures::{Future, Stream};
use futures::future;
use actix_web::{error,HttpRequest, HttpMessage, Error};
use actix_web::multipart::{MultipartItem, Multipart};
use bytes::*;
use self::http::header;

/**
  почему-то в IDE не работает автокомплит после collect
  TODO: убрать после исправления https://github.com/intellij-rust/intellij-rust/issues/3111
*/
fn collect<S: Stream>(s: S) -> impl Future<Item=Vec<S::Item>, Error=S::Error> {
    s.collect()
}

fn invoke_body_size<M: HttpMessage>(m: &M) -> Result<usize, Error> {
    match m.headers().get(header::CONTENT_LENGTH) {
        None => return Err(error::ErrorBadRequest("Content length must be set")),
        Some(h) => h,
    }.to_str()
        .map_err(error::ErrorBadRequest)
        .and_then(|x| x.parse().map_err(error::ErrorBadRequest))
}

fn read_multipart<S>(multipart: Multipart<S>, size: usize) -> impl Future<Item=Bytes, Error=Error>
    where S: Stream<Item=Bytes, Error=error::PayloadError> {
    let stream = multipart
        .from_err()
        .filter_map(|item| match item {
            MultipartItem::Field(f) => Some(f),
            MultipartItem::Nested(_) => unimplemented!("nested multipart"),
        })
        .and_then(move |field| {
            field.fold(BytesMut::with_capacity(size), |mut buf, chunk| {
                buf.put(chunk);
                future::ok::<BytesMut, error::MultipartError>(buf)
            })
        })
        .map(|b| Bytes::from(b))
        .from_err();
    collect(stream).map(|v| { //как еще получить владение на первый элемент вектора?
        for b in v {
            return b;
        }
        unreachable!()
    })
}

pub fn invoke_file(req: &HttpRequest) -> impl Future<Item=Bytes, Error=Error> {
    let multipart = req.multipart();
    future::done(invoke_body_size(req)).and_then(move |size| {
        read_multipart(multipart, size)
    })
}