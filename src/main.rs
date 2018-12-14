extern crate actix_web;
extern crate bip_metainfo;
extern crate bip_util;
extern crate bytes;
extern crate futures_fs;
extern crate futures;
extern crate uuid;
extern crate hex;
extern crate failure;

#[macro_use] extern crate failure_derive;
#[macro_use] extern crate serde_derive;

mod request_utils;
mod storage;
mod response;

use actix_web::{
    Error,
    server,
    App,
    AsyncResponder,
    FutureResponse,
    HttpRequest,
    HttpResponse,
    Responder,
    http::Method,
};
use futures::{Future, Stream};
use bip_metainfo::MetainfoFile;
use storage::CachedSink;
use response::TorrentFile;


fn index(_req: &HttpRequest) -> impl Responder {
    HttpResponse::Ok().body(r#"<html>
        <head><title>Upload Test</title></head>
        <body>
            <form action="/torrent" method="POST" enctype="multipart/form-data">
                <input type="file" name="file"/>
                <input type="submit" value="Submit"></button>
            </form>
        </body>
    </html>"#
    )
}


fn upload_torrent(req: HttpRequest) -> FutureResponse<HttpResponse> {
    match request_utils::invoke_body_size(&req) {
        Err(err) => futures::failed(err).responder(),
        Ok(size) => {
            use futures::sink::Sink;
            use uuid::Uuid;
            let file_name = Uuid::new_v4().to_string();
            request_utils::invoke_request_data(&req)
                .forward(
                    CachedSink::new(
                        storage::make_writer(file_name.to_string()), //to_string - неявный clone
                        size,
                    )
                )
                .and_then(move |(_, sink)| {
                    let bytes = sink.as_ref();
                    let metainfo = MetainfoFile::from_bytes(bytes).unwrap();
                    let response = TorrentFile::from(&metainfo);
                    match serde_json::to_string(&response) {
                        Ok(body) => Ok(HttpResponse::Ok().body(body).into()),
                        Err(e) => unimplemented!(),
                    }
                }).responder()
        }
    }
}

fn main() {
    server::new(||
        vec![
            App::new()
                .resource("/", |r| r.f(index))
                .route("/torrent", Method::POST, upload_torrent)
        ])
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}