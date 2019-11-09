use failure::Fail;
use super::message::{PeerMessage, Handshake, Bitfield};
use super::parser;
use bytes::{Bytes};
use std::net::SocketAddr;
use async_std::prelude::*;
use async_std::net::TcpStream;
use std::io;

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
    fn from(e: io::Error) -> Self {
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
    pub async fn new(mut stream: TcpStream, handshake: Handshake) -> Result<Self, PeerError> {
        let mut bytes: Bytes = handshake.clone().into();
        stream.write_all(bytes.as_ref()).await?;
        let response = parser::read_handshake(&mut stream).await?;
        handshake.validate(&response);
        let bytes: Bytes = PeerMessage::Interested.into();
        stream.write_all(bytes.as_ref()).await?;
        Ok(Peer {
            channel: stream,
            bitfield: vec![],
            state: (PeerState::Unchocked, PeerState::Chocked)
        })
    }
    pub fn have(&self, piece: u32) -> bool {
        self.bitfield.have_bit(piece)
    }
}

/*
struct Connection<R>(Box<Future<Item=(R, PeerMessage), Error=io::Error>>);

impl<R: 'static + AsyncRead> Stream for Connection<R> {
    type Item = PeerMessage;
    type Error = PeerError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let Self(fut) = self;
        let (read, message) = try_ready!(fut.poll());
        let new_fut = super::parser::read_message(read);
        self.0 = Box::new(new_fut);
        Ok(Async::Ready(Some(message)))
    }
}
*/