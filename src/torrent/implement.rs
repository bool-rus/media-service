

use bip_metainfo::MetainfoFile;
use super::*;
use actix_web::client;
use futures::future::Future;
use actix_web::HttpMessage;
use bytes::Bytes;


pub struct TorrentClient {
    meta: MetainfoFile
}


impl TorrentClient {
    pub fn new(meta: MetainfoFile) -> TorrentClient {
        let announce = meta.main_tracker()
            .ok_or(TorrentError("announce url not found in .torrent file".to_string())).unwrap().to_string();
        let info_hash = percent_encoding::percent_encode(meta.info_hash().as_ref(), percent_encoding::PATH_SEGMENT_ENCODE_SET).to_string();
        let peer_id = "12345678901234567890";
        let port = 6882;
        let uploaded = 0;
        let downloaded = 0;
        let left = 3506438144u64;
        let event = "started";
        let uri = format!("{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&event={}",
                          announce,
                          info_hash,
                          peer_id,
                          port,
                          uploaded,
                          downloaded,
                          left,
                          event
        );
        let _tracker_response = client::get(uri)   // <- Create request builder
        .header("User-Agent", "Actix-web")
            .finish().unwrap()
            .send()                               // <- Send http request
            .map_err(|e| panic!("Error: {:?}", e))
            .and_then(|response| response.body())
            .map_err(|e|panic!("Error: {:?}", e));
        TorrentClient {
            meta
        }
    }
}


impl faces::TorrentClient for TorrentClient {
    fn download(&mut self) -> SizedStream {

        let stream = futures::stream::once(Ok(Bytes::from("bugoga")));
        SizedStream::new(0,stream)
    }

    fn download_file(&mut self, num: usize) -> SizedStream {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use actix_web::actix;
    use actix_web::client;
    use futures::future::Future;

    #[test]
    fn test_client() {}
}