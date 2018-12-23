use super::HashString;
use bytes::{Bytes, BytesMut, BufMut};

type TorrentExtentions = [u8;8];

struct Bitfield;

impl Bitfield {
    fn bytes(&self) -> &[u8] {
        unimplemented!()
    }
}

const SIZE_BYTES: usize = 4;
const PORT_BYTES: usize = 2;

pub struct Handshake {
    protocol: String,
    extentions: TorrentExtentions,
    info_hash: HashString,
    peer_id: HashString,
}

impl Into<Bytes> for Handshake {
    fn into(self) -> Bytes {
        let protocol = self.protocol.as_bytes();
        let size = protocol.len() + self.extentions.len() + self.info_hash.len() + self.peer_id.len();
        let mut ret = BytesMut::with_capacity(SIZE_BYTES + size);
        let size = size as u32;
        ret.put_u32_be(size);
        ret.put(protocol);
        ret.put(self.extentions.as_ref());
        ret.put(self.info_hash.as_ref());
        ret.put(self.peer_id.as_ref());
        ret.into()
    }
}

pub enum PeerMessage {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have(u32),
    Bitfield(Bitfield),
    Request {
        block: u32,
        offset: u32,
        length: u32
    },
    Piece {
        block: u32,
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

fn make_empty_message(message_id: u8) -> Bytes {
    let size = 1;
    let mut ret = BytesMut::with_capacity(size + SIZE_BYTES);
    let size = size as u32;
    ret.put_u32_be(size);
    ret.put(message_id);
    ret.into()
}

impl Into<Bytes> for PeerMessage { //TODO: может, лучше в Stream?
    fn into(self) -> Bytes {
        match self {
            PeerMessage::KeepAlive => Bytes::from([0u8,0u8,0u8,0u8].as_ref()),
            PeerMessage::Choke => make_empty_message(b'0'),
            PeerMessage::Unchoke => make_empty_message(b'1'),
            PeerMessage::Interested => make_empty_message(b'2'),
            PeerMessage::NotInterested => make_empty_message(b'3'),
            PeerMessage::Have(index) => {
                let size = 1 + SIZE_BYTES;
                let mut ret = BytesMut::with_capacity(size + SIZE_BYTES);
                ret.put_u32_be(size as u32);
                ret.put(b'4');
                ret.put_u32_be(index);
                ret.into()
            },
            PeerMessage::Bitfield(bitfield) => {
                let body = bitfield.bytes();
                let size = 1 + body.len();
                let mut ret = BytesMut::with_capacity(size + SIZE_BYTES);
                ret.put_u32_be(size as u32);
                ret.put(b'5');
                ret.put(body);
                ret.into()
            },
            PeerMessage::Request { block, offset, length } => {
                let size = 1 + 3*SIZE_BYTES;
                let mut ret = BytesMut::with_capacity(size + SIZE_BYTES);
                ret.put_u32_be(size as u32);
                ret.put(b'6');
                ret.put_u32_be(block);
                ret.put_u32_be(offset);
                ret.put_u32_be(length);
                ret.into()
            },
            PeerMessage::Piece { block, offset, data } => {
                let size = 1 + 2*SIZE_BYTES + data.len();
                let mut ret = BytesMut::with_capacity(size + SIZE_BYTES);
                ret.put_u32_be(size as u32);
                ret.put(b'7');
                ret.put_u32_be(block);
                ret.put_u32_be(offset);
                ret.put(data);
                ret.into()
            },
            PeerMessage::Cancel { block, offset, length } => {
                let size = 1 + 3*SIZE_BYTES;
                let mut ret = BytesMut::with_capacity(size + SIZE_BYTES);
                ret.put_u32_be(size as u32);
                ret.put(b'6');
                ret.put_u32_be(block);
                ret.put_u32_be(offset);
                ret.put_u32_be(length);
                ret.into()
            },
            PeerMessage::Port(port) => {
                let size = 1 + PORT_BYTES;
                let mut ret = BytesMut::with_capacity(size + SIZE_BYTES);
                ret.put_u32_be(size as u32);
                ret.put(b'9');
                ret.put_u16_be(port);
                ret.into()
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::PeerMessage;
    use bytes::Bytes;

    #[test]
    fn test_empty_messages() {
        let bytes: Bytes = PeerMessage::KeepAlive.into();
        assert_eq!([0u8,0,0,0].as_ref(), bytes.as_ref());
        let bytes: Bytes = PeerMessage::Choke.into();
        assert_eq!([0u8,0,0,1,b'0'].as_ref(), bytes.as_ref());
        let bytes: Bytes = PeerMessage::Unchoke.into();
        assert_eq!([0u8,0,0,1,b'1'].as_ref(), bytes.as_ref());
        let bytes: Bytes = PeerMessage::Interested.into();
        assert_eq!([0u8,0,0,1,b'2'].as_ref(), bytes.as_ref());
        let bytes: Bytes = PeerMessage::NotInterested.into();
        assert_eq!([0u8,0,0,1,b'3'].as_ref(), bytes.as_ref());
    }

    #[test]
    fn test_simple_messages() {
        let bytes: Bytes = PeerMessage::Have(0x342f21cc).into();
        assert_eq!([0u8,0,0,5,b'4',0x34,0x2f,0x21,0xcc].as_ref(), bytes.as_ref());
    }
}