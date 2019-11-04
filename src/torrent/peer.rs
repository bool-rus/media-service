use super::tokio::net::TcpStream;
use super::tokio_io::{AsyncRead, AsyncWrite};
use super::tokio::io;
use failure::Fail;
use torrent::message::{PeerMessage, Handshake, Bitfield};
use bytes::{Bytes, IntoBuf};
use futures::Future;
use std::net::SocketAddr;
extern crate byteorder;
use self::byteorder::{BigEndian, ReadBytesExt};


#[derive(Debug,Fail)]
#[fail(display="{}",0)]
pub struct PeerError(pub String);

enum PeerState {
    Chocked,
    Unchocked,
}

pub struct Peer {
    channel: TcpStream,
    bitfield: Vec<u8>,
    state: (PeerState, PeerState),
}

impl Peer {
    pub fn new(addr: SocketAddr, handshake: Handshake) -> impl Future<Item=Self,Error=io::Error> {
        let handshake_request = handshake.clone();
        TcpStream::connect(&addr).and_then( |stream|{
            let bytes: Bytes = handshake.into();
            io::write_all(stream, bytes)
        }).and_then(|(mut stream, _)|{
            Handshake::parse(stream)
        }).and_then(move |(handshake_response, stream)| {
            assert!(handshake_request.validate(&handshake_response));
            let bytes: Bytes = PeerMessage::Interested.into();
            io::write_all(stream,bytes)
        }).and_then(|(stream, _)| {
            Ok(Peer {
                channel: stream,
                bitfield: vec![],
                state: (PeerState::Unchocked, PeerState::Chocked)
            })
        })
    }
    pub fn have(&self, piece: u32) -> bool {
        self.bitfield.have_bit(piece)
    }
}

