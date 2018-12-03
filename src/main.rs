extern crate actix_web;
extern crate bip_metainfo;
extern crate bytes;
extern crate futures_fs;
extern crate futures;

mod request_utils;

use actix_web::{
    server,
    App,
    AsyncResponder,
    FutureResponse,
    HttpRequest,
    HttpResponse,
    Responder,
    http::Method,
    dev::Handler,
};
use futures::Future;
use bip_metainfo::MetainfoFile;

/// use actix_web::{
///     AsyncResponder, FutureResponse, HttpMessage, HttpRequest, HttpResponse,
/// };
/// use bytes::Bytes;
/// use futures::future::Future;



fn index(_req: &HttpRequest) -> impl Responder {
    HttpResponse::Ok().body( r#"<html>
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


struct HandlerImpl;
impl<S> Handler<S> for HandlerImpl {
    type Result = HttpResponse;

    fn handle(&self, req: &HttpRequest<S>) -> HttpResponse {
        format!("bgg, path: {}", req.path()).into()
    }
}



fn upload_torrent(req: HttpRequest) -> FutureResponse<HttpResponse> {

    request_utils::invoke_file(&req).and_then(|bytes| {
        println!("received bytes: {:?}", bytes.len());
        println!("received text: {:?}", bytes);
        let metainfo = MetainfoFile::from_bytes(bytes).unwrap();
        let files: Vec<_> = metainfo.info().files()
            .filter_map(|it| it.path().to_str()).collect();
        let res = HttpResponse::Ok().body(files.join("\n"));
        Ok(res.into())
    }).responder()
}

fn main() {
    server::new(||
        vec![
            //App::new().resource("/", |r| r.f(index)),
            App::new()
                .resource("/", |r|r.f(index))
                .handler("/video", HandlerImpl)
                .route("/torrent", Method::POST, upload_torrent)
        ])
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}