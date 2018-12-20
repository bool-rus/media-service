extern crate bencoders;
extern crate nom;
mod faces;
mod implement;
mod tracker;
pub use self::faces::*;

pub fn new_client(meta: bip_metainfo::MetainfoFile) -> impl TorrentClient {
    implement::TorrentClient::new(meta)
}