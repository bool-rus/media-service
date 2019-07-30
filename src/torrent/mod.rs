extern crate tokio_core;

mod faces;
mod implement;
mod tracker;
mod peer;
pub use self::faces::*;

pub fn new_client(meta: bip_metainfo::MetainfoFile) -> impl TorrentClient {
    implement::TorrentClient::new(meta)
}