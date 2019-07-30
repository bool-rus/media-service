

use bip_metainfo::MetainfoFile;
use super::*;
use actix_web::client;
use futures::future::Future;
use actix_web::HttpMessage;
use bytes::Bytes;
use futures::sync::mpsc;
use futures::sync::mpsc::{Sender, Receiver};
use futures::Stream;
use futures::Async;
use self::tokio_core::reactor::Core;
use self::tokio_core::net::TcpStream;
use std::collections::HashMap;
use bip_metainfo::InfoHash;
use std::net::IpAddr;
use std::cell::RefCell;
use futures::sink::Sink;
use std::mem;
use std::collections::VecDeque;

struct Block;

struct TorrentService {
    peer_id: HashString,
    uploaded: u64,
    downloaded: u64,
    torrents: HashMap<InfoHash,TorrentConnection>,
}


fn new_service() -> Sender<TorrentRequest> {
    let service = RefCell::new(TorrentService::new());
    let (s,r) = mpsc::channel::<TorrentRequest>(100);
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let runner = r.for_each(|req|{
        use self::tracker::*;
        let handle = handle.clone();
        let TorrentRequest{ meta, filenum: file, sender, receiver} = req;

        let processor = receiver.for_each(move |_| {
            let sender = sender.clone();

            futures::future::ok(())
        });
        handle.spawn(processor);
        futures::future::ok(())
    });
    core.run(runner).unwrap();
    s
}

impl TorrentService {
    fn new() -> Self {
        unimplemented!()
    }
    fn new_torrent(&mut self, req: &TorrentRequest) {
        let meta = &req.meta;
        let announce = meta.main_tracker()
            .ok_or(TorrentError("announce url not found in .torrent file".to_string())).unwrap().to_string();
        let info_hash = percent_encoding::percent_encode(meta.info_hash().as_ref(), percent_encoding::PATH_SEGMENT_ENCODE_SET).to_string();
        let peer_id = percent_encoding::percent_encode(&self.peer_id, percent_encoding::PATH_SEGMENT_ENCODE_SET).to_string();
        let port = 6882;
        let uploaded = self.uploaded;
        let downloaded = self.downloaded;
        let left = 99999; //TODO: надо вытащить размер файла из меты
        let event = "started";
        let uri = format!("{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&event={}",
                          announce,
                          info_hash,
                          peer_id,
                          port,
                          uploaded,
                          downloaded,
                          left,
                          event
        );
        let _tracker_response = client::get(uri)   // <- Create request builder
            .header("User-Agent", "Actix-web")
            .finish().unwrap()
            .send()                               // <- Send http request
            .map_err(|e| panic!("Error: {:?}", e))
            .and_then(|response| response.body())
            .map_err(|e|panic!("Error: {:?}", e));
    }
}

pub struct TorrentClient {

}

enum PeerState {
    Chocked,
    Unchocked,
}

struct Peer {
    channel: TcpStream,
    bitfield: Vec<u8>,
    state: (PeerState, PeerState),
}

struct TorrentRequest {
    meta: MetainfoFile,
    filenum: usize, //номер файла в торрент-файле
    sender: Sender<Bytes>,
    receiver: Receiver<()>, // когда у нас дернется receiver нужно будет послать байты в sender
}

struct TorrentConnection {
    client: RefCell<TorrentService>,
    request: TorrentRequest,
    peers: Vec<IpAddr>,
    connections: Vec<Peer>,
    cache: VecDeque<Bytes>,
}

impl TorrentConnection {
    //три сервиса:
    // один перидочески обращается к торрент-трекеру и обрабатывает его ответы,
    // второй обрабатывает ответы от присоединенных пиров,
    // третий обрабатывает
    fn update_torrent_info(&mut self) {

    }
    fn process_peer(&mut self) {

    }
    fn process_download(&mut self) {
        let cache = self.cache.pop_front().unwrap();
        self.request.sender.try_send(cache).unwrap();

    }

}


struct TorrentStream {
    sender: Sender<()>,
    receiver: Receiver<Bytes>
}

impl Stream for TorrentStream {
    type Item = Bytes;
    type Error = ();

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        match self.receiver.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),
            ret @ Ok(Async::Ready(None)) => ret, //в каком случае придет none?
            ret @ Ok(Async::Ready(Some(_))) => {
                self.sender.try_send(()).unwrap();
                ret
            }
            err @ Err(_) => err,
        }
    }
}


impl TorrentClient {
    pub fn new(meta: MetainfoFile) -> Self {
        unimplemented!();
    }
}


impl faces::TorrentClient for TorrentClient {
    fn download(&mut self) -> SizedStream {

        let stream = futures::stream::once(Ok(Bytes::from("bugoga")));
        SizedStream::new(0,stream)
    }

    fn download_file(&mut self, num: usize) -> SizedStream {

        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use actix_web::actix;
    use actix_web::client;
    use futures::future::Future;

    #[test]
    fn test_client() {}
}