
use futures::{Future, Stream};
use futures::future;
use actix_web::{HttpRequest, HttpMessage};
use bytes::*;
use actix_web::error;
use actix_web::multipart::MultipartItem;

/**
  почему-то в IDE не работает автокомплит после collect
  TODO: убрать после исправления https://github.com/intellij-rust/intellij-rust/issues/3111
*/
fn collect<S: Stream>(s: S) -> impl Future<Item=Vec<S::Item>, Error=S::Error> {
    s.collect()
}

pub fn invoke_file(req: &HttpRequest) -> impl Future<Item=Bytes, Error=actix_web::Error> {
    let stream = req.multipart().from_err()
        .filter_map(|item| match item {
            MultipartItem::Field(f) => Some(f),
            MultipartItem::Nested(_) => None,
        })
        .and_then(|field| {
            field.fold(BytesMut::with_capacity(512*1024), |mut buf, chunk| {
                buf.put(chunk);
                future::ok::<BytesMut,error::MultipartError>(buf)
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