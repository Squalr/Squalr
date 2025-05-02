use std::path::PathBuf;

pub struct AppProvisionerConfig {}

impl AppProvisionerConfig {
    pub const DOWNLOAD_WEIGHT: f32 = 0.35;
    pub const EXTRACT_WEIGHT: f32 = 1.0 - AppProvisionerConfig::DOWNLOAD_WEIGHT;
    pub const FILENAME: &'static str = "Squalr.zip";
    const GITHUB_API_LATEST_RELEASE: &'static str = "https://api.github.com/repos/zcanann/Squalr-Rust/releases/latest";

    /// Gets the version URL for the latest release.
    pub fn get_latest_version_url() -> &'static str {
        &AppProvisionerConfig::GITHUB_API_LATEST_RELEASE
    }

    pub fn get_default_install_dir() -> anyhow::Result<PathBuf> {
        let mut install_dir = dirs::data_local_dir().ok_or_else(|| anyhow::anyhow!("Failed to get local app data directory"))?;
        install_dir.push("Programs");
        install_dir.push("Squalr");
        Ok(install_dir)
    }
}
