use super::HashString;
use bytes::Bytes;

type TorrentExtentions = [u8;8];
struct Bitfield;

pub struct Handshake {
    protocol: String,
    extentions: TorrentExtentions,
    info_hash: HashString,
    peer_id: HashString,
}

pub enum PeerMessage {
    KeepAlive,
    Choke,
    Unchoke,
    Interested(bool),
    Have(u32),
    Bitfield(Bitfield),
    Request {
        block: u32,
        offset: u32,
        length: u32
    },
    Piece {
        index: u32,
        offset: u32,
        data: Bytes,
    },
    Cancel { //зачем это?
        block: u32,
        offset: u32,
        length: u32
    },
    Port(u16), //unimplemented
}