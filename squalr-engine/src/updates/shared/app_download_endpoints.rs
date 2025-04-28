pub struct AppDownloadEndpoints {}

impl AppDownloadEndpoints {
    pub const DOWNLOAD_WEIGHT: f32 = 0.35;
    pub const EXTRACT_WEIGHT: f32 = 1.0 - AppDownloadEndpoints::DOWNLOAD_WEIGHT;
    pub const FILENAME: &'static str = "Squalr.zip";
    const GITHUB_API_LATEST_RELEASE: &'static str = "https://api.github.com/repos/zcanann/Squalr-Rust/releases/latest";

    /// Gets the version URL for the latest release.
    pub fn get_latest_version_url() -> &'static str {
        &AppDownloadEndpoints::GITHUB_API_LATEST_RELEASE
    }
}
