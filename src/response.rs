use bip_metainfo::MetainfoFile;

#[derive(Serialize, Deserialize)]
pub struct TorrentFile {
    hash: String,
    announce: Option<String>,
    comment: Option<String>,
    files: Vec<String>
}

impl From<&MetainfoFile> for TorrentFile {
    fn from(meta: &MetainfoFile) -> Self {
        TorrentFile {
            hash: hex::encode(meta.info_hash()),
            announce: meta.main_tracker().map(ToString::to_string),
            comment: meta.comment().map(ToString::to_string),
            files: meta.info().files()
                .map(|f|f.path().to_str().unwrap_or("?").to_string())
                .collect()
        }
    }
}
