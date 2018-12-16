extern crate bip_utracker;
use bip_metainfo::MetainfoFile;
use super::*;
use self::bip_utracker::TrackerClient;
use actix_web::actix;
use actix_web::client;
use futures::future::Future;

pub struct TorrentClient {
    meta: MetainfoFile,
    tracker: TrackerClient
}

impl TorrentClient {
    pub fn new(meta: MetainfoFile) -> Result<Self, TorrentError> {
        let announce = meta.main_tracker()
            .ok_or(TorrentError("announce url not found in .torrent file".to_string()))?;
        let hash = meta.info_hash();
        actix::run(
            || client::get("http://www.rust-lang.org")   // <- Create request builder
                .header("User-Agent", "Actix-web")
                .finish().unwrap()
                .send()                               // <- Send http request
                .map_err(|_| panic!())
                .and_then(|response| {                // <- server http response
                    println!("Response: {:?}", response);
                    Ok(())
                })
        );
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use torrent::implement::TorrentClient;

    #[test]
    fn test_client() {

    }
}