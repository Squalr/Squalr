pub struct AppDownloadEndpoints {}

impl AppDownloadEndpoints {
    pub const DOWNLOAD_WEIGHT: f32 = 0.35;
    pub const EXTRACT_WEIGHT: f32 = 1.0 - AppDownloadEndpoints::DOWNLOAD_WEIGHT;
    pub const BUCKET_NAME: &'static str = "squalr-134d6.appspot.com";
    pub const UPDATES_PREFIX: &'static str = "releases/windows";
    pub const FILENAME: &'static str = "Squalr.zip";

    /// Gets the download URL for the latest release.
    pub fn get_latest_download_url() -> String {
        format!(
            "https://firebasestorage.googleapis.com/v0/b/{}/o/{}%2F{}?alt=media",
            Self::BUCKET_NAME,
            urlencoding::encode(Self::UPDATES_PREFIX),
            urlencoding::encode(Self::FILENAME)
        )
    }
}
