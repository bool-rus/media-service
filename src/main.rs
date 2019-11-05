extern crate actix_web;
extern crate bip_metainfo;
extern crate bytes;
extern crate futures_fs;
extern crate futures;
extern crate uuid;
extern crate hex;
extern crate failure;

#[macro_use] extern crate failure_derive;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate display_derive;
extern crate core;

mod request_utils;
mod storage;
mod response;
mod torrent;

use actix_web::{
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
use actix_web::Body;


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
                        Err(_e) => unimplemented!(),
                    }
                }).responder()
        }
    }
}

fn download(req: HttpRequest) -> FutureResponse<HttpResponse> {
    let hash = req.query().get("hash").unwrap().to_string();
    use torrent::*;
    storage::read(hash)
        .from_err()
        .map(|bytes| MetainfoFile::from_bytes(bytes).unwrap()) //result -to future
        .and_then(move |meta | {
            let mut client = torrent::new_client(meta);
            let body = Box::new(client.download().from_err());

            Ok(req.build_response(Default::default())
                .chunked()
                .body(Body::Streaming(body)).into())
        })
        .responder()
}

fn main() {
    server::new(||
        vec![
            App::new()
                .resource("/", |r| r.f(index))
                .route("/torrent", Method::POST, upload_torrent)
                .route("/torrent/download", Method::GET, download)
        ])
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}