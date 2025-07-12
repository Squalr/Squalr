#[derive(Clone)]
pub struct DownloadProgress {
    pub filename: String,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
}
