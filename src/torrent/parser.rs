extern crate nom;
use self::nom::{
    IResult,
    bytes::complete::*,
    number::complete::*,
    character::complete::anychar,
    sequence::tuple
};
use super::message::*;
use super::HashString;
use tokio::prelude::*;
use bytes::{Bytes, BytesMut, BufMut};

fn parse_hash_string(i: &[u8]) -> IResult<&[u8], HashString> {
    let (i, slice) = take(20usize)(i)?;
    let mut res: [u8;20] = Default::default();
    res.copy_from_slice(slice);
    Ok((i, res))
}

fn parse_torrent_extentions(i: &[u8]) -> IResult<&[u8], TorrentExtentions> {
    let (i, slice) = take(8usize)(i)?;
    let mut res: [u8;8] = Default::default();
    res.copy_from_slice(slice);
    Ok((i, res))
}

fn parse_handshake(i: &[u8], size: u8) -> IResult<&[u8], Handshake> {
    let (i, (protocol, extentions, info_hash, peer_id)) = tuple((
        take(size),
        parse_torrent_extentions,
        parse_hash_string,
        parse_hash_string
    ))(i)?;
    Ok((i, Handshake {
        protocol: std::str::from_utf8(protocol).unwrap().to_string(),
        extentions,
        info_hash,
        peer_id
    }))
}

#[test]
fn test_parse_handshake() {
    let x = Handshake {
        protocol: "bugoga".to_string(),
        extentions: [1u8, 2, 3, 4, 5, 6, 7, 8],
        info_hash: [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20],
        peer_id: [20u8, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1],
    };
    let bytes: Bytes = x.clone().into();
    let parseRes = parse_handshake(&bytes.as_ref()[1..], 6);
    assert_eq!(Result::Ok((b"".as_ref(), x)), parseRes);
    let (buf, handshake) = parseRes.unwrap();
}

fn parse_message(i: &[u8], size: u32) -> IResult<&[u8],PeerMessage> {
    if size == 0 {
        Ok((i, PeerMessage::KeepAlive))
    } else {
        let (i, tag) = anychar(i)?;
        match tag {
            '0' => Ok((i, PeerMessage::Choke)),
            '1' => Ok((i, PeerMessage::Unchoke)),
            '2' => Ok((i, PeerMessage::Interested)),
            '3' => Ok((i, PeerMessage::NotInterested)),
            '4' => {
                let (i, index) = be_u32(i)?;
                Ok((i, PeerMessage::Have(index)))
            },
            '5' => {
                let (i, bitfield) = take(size - 1)(i)?;
                Ok((i, PeerMessage::Bitfield(bitfield.to_vec())))
            },
            '6' => {
                let (i, (block, offset, length)) = tuple((be_u32, be_u32, be_u32))(i)?;
                Ok((i, PeerMessage::Request {block, offset, length}))
            },
            '7' => {
                let (i, (block, offset, data)) = tuple((be_u32, be_u32, take(size -9)))(i)?;
                Ok((i, PeerMessage::Piece{block, offset, data: data.into()}))
            },
            '8' => {
                let (i, (block, offset, length)) = tuple((be_u32, be_u32, be_u32))(i)?;
                Ok((i, PeerMessage::Cancel {block, offset, length}))
            },
            '9' => {
                let (i, port) = be_u16(i)?;
                Ok((i, PeerMessage::Port(port)))
            },
            _ => unreachable!()
        }
    }
}

#[test]
fn test_parse_keep_alive() {
    let val = PeerMessage::KeepAlive;
    let bytes: Bytes = val.clone().into();
    assert_eq!(parse_message(&bytes.as_ref()[4..],0), Ok((b"".as_ref(), val)));
}
#[test]
fn test_parse_interested() {
    let val = PeerMessage::Interested;
    let bytes: Bytes = val.clone().into();
    assert_eq!(parse_message(&bytes.as_ref()[4..], 1), Ok((b"".as_ref(), val)));
}
#[test]
fn test_parse_have() {
    let val = PeerMessage::Have(463234);
    let bytes: Bytes = val.clone().into();
    assert_eq!(parse_message(&bytes.as_ref()[4..], 4), Ok((b"".as_ref(), val)));
}
#[test]
fn test_parse_bitfield() {
    let val = PeerMessage::Bitfield(b"adnfysdfnskdfj".to_vec());
    let bytes: Bytes = val.clone().into();
    assert_eq!(parse_message(&bytes.as_ref()[4..], 15), Ok((b"".as_ref(), val)));
}
#[test]
fn test_parse_request() {
    let val = PeerMessage::Request {block: 12423, offset: 345, length: 13453};
    let bytes: Bytes = val.clone().into();
    assert_eq!(parse_message(&bytes.as_ref()[4..], 13), Ok((b"".as_ref(), val)));
}
#[test]
fn test_parse_piece() {
    let val = PeerMessage::Piece {block: 123,offset:234, data: b"sadnfkydfasdfwefgsdresadnfkybnf".as_ref().into()};
    let bytes: Bytes = val.clone().into();
    assert_eq!(parse_message(&bytes.as_ref()[4..], 40), Ok((b"".as_ref(), val)));
}
#[test]
fn test_parse_cancel() {
    let val = PeerMessage::Cancel {block:31455,offset:12334,length:2355};
    let bytes: Bytes = val.clone().into();
    assert_eq!( parse_message(&bytes.as_ref()[4..], 13), Ok((b"".as_ref(),val)));
}
#[test]
fn test_parse_port() {
    let val = PeerMessage::Port(63445);
    let bytes: Bytes = val.clone().into();
    assert_eq!(parse_message(&bytes.as_ref()[4..], 4), Ok((b"".as_ref(),val)));
}


pub fn read_message<T: AsyncRead>(read: T) -> impl Future<Item=(T, PeerMessage), Error=tokio::io::Error> {
    use tokio::io;
    io::read_exact(read, [0u8;4]).and_then(|(r,buf)| {
        let (_, size) = be_u32::<()>(&buf).unwrap();
        let buf = BytesMut::with_capacity(size as usize);
        io::read_exact(r, buf).map(move |(r,b)|(r,b,size))
    }).and_then(|(read, buf, size)|{
        let (_, message) = parse_message(&buf, size).unwrap();
        Ok((read, message))
    })
}

pub fn read_handshake<T: AsyncRead>(read: T) -> impl Future<Item=(T,Handshake), Error=tokio::io::Error> {
    use tokio::io;
    io::read_exact(read, [0u8]).and_then(|(r, [protocol_size])|{
        let buf = BytesMut::with_capacity(super::message::HANDSHAKE_DEFAULT_SIZE - 1 + protocol_size as usize);
        io::read_exact(r, buf).map(move|(r,b)|(r,b,protocol_size))
    }).and_then(|(r, buf, protocol_size)|{
        let (_, handshake) = parse_handshake(&buf, protocol_size).unwrap();
        Ok((r, handshake))
    })
}
