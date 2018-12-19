use super::bencoders::*;
use super::nom::IResult;
use std::net::IpAddr;
use bytes::Bytes;
use std::collections::HashMap;
use std::marker::PhantomData;

type HashString = [u8;20];

#[derive(Debug, Display)]
pub enum TrackerEvent {
    #[display(fmt="started")]
    Started,
    #[display(fmt="stopped")]
    Stopped,
    #[display(fmt="completed")]
    Completed
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
    numwant: Option<usize>, //default 50
    key: Option<String>,
    tracker_id: Option<String>,

}

pub struct Peer {
    id: Option<HashString>,
    ip: IpAddr,
    port: u16
}

pub enum AnnounceResponseError {
    FailureMessage(String),
    Incorrect
}
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

impl From<Bytes> for AnnounceResponse {
    fn from(bytes: Bytes) -> Self {
        match bencoders::decode(bytes.as_ref()){
            IResult::Done(_, Bencode::Dict(dict)) => {
                if let Some(Bencode::Bytes(reason)) = dict.get(b"failure reason".as_ref()) {
                    let err  = AnnounceResponseError::FailureMessage(
                        String::from_utf8(reason.clone()).unwrap()
                    );
                    AnnounceResponse::Failure(err)
                } else {
                    response_from_dict(&dict).unwrap_or(
                        AnnounceResponse::Failure(AnnounceResponseError::Incorrect)
                    )
                }
            },
            _ => AnnounceResponse::Failure(AnnounceResponseError::Incorrect),
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
            },
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
        peers: Vec::new() //TODO: Распарсить список пиров
    })
}