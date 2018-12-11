extern crate http;

use futures::Stream;
use actix_web::{error, HttpRequest, HttpMessage, Error};
use actix_web::multipart::{MultipartItem, Multipart};
use bytes::Bytes;
use self::http::header;


pub fn invoke_body_size<M: HttpMessage>(m: &M) -> Result<usize, Error> {
    match m.headers().get(header::CONTENT_LENGTH) {
        None => return Err(error::ErrorBadRequest("Content length must be set")),
        Some(h) => h,
    }.to_str()
        .map_err(error::ErrorBadRequest)
        .and_then(|x| x.parse().map_err(error::ErrorBadRequest))
}

fn read_multipart<S>(multipart: Multipart<S>) -> impl Stream<Item=Bytes, Error=Error>
    where S: Stream<Item=Bytes, Error=error::PayloadError> {
    multipart
        .filter_map(|item| match item {
            MultipartItem::Field(f) => Some(f),
            MultipartItem::Nested(_) => unimplemented!("nested multipart"),
        })
        .flatten()
        .from_err()
}

pub fn invoke_request_data(req: &HttpRequest) -> impl Stream<Item=Bytes, Error=Error> {
    read_multipart(req.multipart())
}