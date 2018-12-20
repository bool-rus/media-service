use super::bencoders::*;
use super::nom::IResult;
use std::net::IpAddr;
use bytes::Bytes;
use std::collections::HashMap;
use super::HashString;

const ADDR_BYTES: usize = 4;
const PORT_BYTES: usize = 2; //note that port is u16, @see invoke_port
const PEER_BYTES: usize = ADDR_BYTES + PORT_BYTES;

fn invoke_port(bytes: &[u8]) -> u16 {
    (bytes[0] as u16) << 8 | (bytes[1] as u16)
}


#[derive(Debug, Display)]
pub enum TrackerEvent {
    #[display(fmt = "started")]
    Started,
    #[display(fmt = "stopped")]
    Stopped,
    #[display(fmt = "completed")]
    Completed,
}

pub struct AnnounceRequest {
    info_hash: HashString,
    peer_id: HashString,
    port: u16,
    uploaded: u64,
    downloaded: u64,
    left: u64,
    //compact: usize, //1 or 0
    //no_peer_id: usize, //1 or 0
    event: TrackerEvent,
    //ip,
    numwant: Option<usize>,
    //default 50
    key: Option<String>,
    tracker_id: Option<String>,

}

#[derive(Debug,PartialEq)]
pub struct Peer {
    id: Option<HashString>,
    ip: IpAddr,
    port: u16,
}

#[derive(Debug, Fail, PartialEq)]
pub enum AnnounceResponseError {
    #[fail(display = "received error message from tracker: {}", 0)]
    FailureMessage(String),
    #[fail(display = "received invalid response from tracker")]
    Invalid,
}

#[derive(Debug, PartialEq)]
pub enum AnnounceResponse {
    Success {
        warning_message: Option<String>,
        interval: usize,
        min_interval: Option<usize>,
        tracker_id: Option<String>,
        complete: usize,
        incomplete: usize,
        peers: Vec<Peer>,
    },
    Failure(AnnounceResponseError),
}

impl From<Bytes> for AnnounceResponse { //TODO: реализовать scrape
    fn from(bytes: Bytes) -> Self {
        match bencoders::decode(bytes.as_ref()) {
            IResult::Done(_, Bencode::Dict(dict)) => {
                if let Some(Bencode::Bytes(reason)) = dict.get(b"failure reason".as_ref()) {
                    let err = AnnounceResponseError::FailureMessage(
                        String::from_utf8(reason.clone()).unwrap()
                    );
                    AnnounceResponse::Failure(err)
                } else {
                    response_from_dict(&dict).unwrap_or(
                        AnnounceResponse::Failure(AnnounceResponseError::Invalid)
                    )
                }
            }
            _ => AnnounceResponse::Failure(AnnounceResponseError::Invalid),
        }
    }
}

trait Translator<T> {
    fn translate(self) -> Option<T>;
}

impl Translator<usize> for Option<&Bencode> {
    fn translate(self) -> Option<usize> {
        match self? {
            &Bencode::Int(val) => Some(val as usize),
            _ => None
        }
    }
}

impl Translator<String> for Option<&Bencode> {
    fn translate(self) -> Option<String> {
        match self? {
            &Bencode::Bytes(ref val) => {
                match String::from_utf8(val.to_vec()) {
                    Ok(val) => Some(val),
                    Err(_) => None,
                }
            }
            _ => None
        }
    }
}

fn response_from_dict(dict: &HashMap<Vec<u8>, Bencode>) -> Option<AnnounceResponse> {
    Some(AnnounceResponse::Success {
        warning_message: dict.get(b"warning message".as_ref()).translate(),
        interval: dict.get(b"interval".as_ref()).translate()?,
        min_interval: dict.get(b"min interval".as_ref()).translate(),
        tracker_id: dict.get(b"tracker id".as_ref()).translate(),
        complete: dict.get(b"complete".as_ref()).translate()?,
        incomplete: dict.get(b"interval".as_ref()).translate()?,
        peers: invoke_peers(dict.get(b"peers".as_ref())?)?, //TODO: Распарсить список пиров
    })
}
fn invoke_peers(bencode: &Bencode) -> Option<Vec<Peer>> {
    match bencode {
        Bencode::Bytes(bytes) => {
            if bytes.len() % PEER_BYTES != 0 {
                return None;
            }
            let mut peers = Vec::with_capacity(bytes.len()/PEER_BYTES);
            let mut slice = bytes.as_slice();
            while slice.len() > 0 {
                let (peer, new_slice) = slice.split_at(PEER_BYTES);
                slice = new_slice;
                let (addr, port) = peer.split_at(ADDR_BYTES);
                let mut arr: [u8;ADDR_BYTES] = Default::default();
                arr.copy_from_slice(addr);
                peers.push(Peer {
                    id: None,
                    ip: IpAddr::from(arr),
                    port: invoke_port(port),
                })
            }
            Some(peers)
        },
        Bencode::Dict(_) => unimplemented!(),
        _ => None,
    }
}


#[test]
fn test_parse() {
    use std::fs::File;
    use std::io::Read;
    let mut bytes = Vec::new();
    File::open("test/announce-response.bencode").unwrap().read_to_end(&mut bytes).unwrap();
    let bytes = Bytes::from(bytes);
    assert_eq!(
        AnnounceResponse::Success {
            warning_message: None,
            interval: 2627,
            min_interval: Some(1313),
            tracker_id: None,
            complete: 30,
            incomplete: 2627,
            peers: vec![
                Peer {id: None, ip: [97,51,102,120].into(), port: 8498},
                Peer {id: None, ip: [98,53,105,100].into(), port: 8257},
            ]
        }, bytes.into()
    );
}
#[test]
fn test_slice() {
    let mut slice = b"bugogablablazazaza".as_ref();
    let check = vec![
        b"bugoga",
        b"blabla",
        b"zazaza"
    ];
    let mut buf = Vec::new();
    while slice.len() > 0 {
        let (chunk, slice2) = slice.split_at(6);
        slice = slice2;
        buf.push(chunk);
    }
    assert_eq!(check,buf);
}