
use std::net::IpAddr;

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
pub struct AnnounceResponse {
    warning_message: Option<String>,
    interval: usize,
    min_interval: Option<usize>,
    tracker_id: Option<String>,
    complete: usize,
    incomplete: usize,
    peers: Vec<Peer>,

}