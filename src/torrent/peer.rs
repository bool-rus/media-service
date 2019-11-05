use tokio::net::TcpStream;
use tokio::io;
use failure::Fail;
use torrent::message::{PeerMessage, Handshake, Bitfield};
use bytes::{Bytes};
use std::net::SocketAddr;

use tokio::io::Error;
use tokio::prelude::*;


#[derive(Debug,Fail)]
pub enum PeerError {
    #[fail(display="{}",0)]
    IoError(io::Error),
    #[fail(display="{}",0)]
    Simple(String),
    #[fail(display="Handshake error")]
    Handshake,
}
impl From<io::Error> for PeerError {
    fn from(e: Error) -> Self {
        PeerError::IoError(e)
    }
}

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
    pub fn new(addr: SocketAddr, handshake: Handshake) -> impl Future<Item=Self,Error=PeerError> {
        let handshake_request = handshake.clone();
        TcpStream::connect(&addr).and_then( |stream| {
            let bytes: Bytes = handshake.into();
            io::write_all(stream, bytes)
        }).and_then(|(stream, _)| {
            Handshake::parse(stream)
        }).from_err().and_then(move |(stream, handshake_response)| {
            if handshake_request.validate(&handshake_response) {
                future::ok(stream)
            } else {
                future::err(PeerError::Handshake)
            }
        }).and_then( |stream| {
            let bytes: Bytes = PeerMessage::Interested.into();
            io::write_all(stream, bytes).from_err()
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

struct Connection(TcpStream);

impl Stream for Connection {
    type Item = PeerMessage;
    type Error = PeerError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        unimplemented!()
    }
}