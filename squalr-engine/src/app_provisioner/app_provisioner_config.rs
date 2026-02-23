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

    /// Resolves the current release artifact target in `<os>-<arch>` format.
    pub fn get_release_artifact_target() -> Option<String> {
        let operating_system_label = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            return None;
        };

        let architecture_label = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            return None;
        };

        Some(format!("{operating_system_label}-{architecture_label}"))
    }

    /// Resolves the expected desktop release bundle file name for a given release tag.
    pub fn get_release_bundle_asset_name(release_tag_name: &str) -> Option<String> {
        let release_version = release_tag_name.trim_start_matches('v');
        Self::get_release_artifact_target().map(|artifact_target| format!("squalr-{release_version}-{artifact_target}.zip"))
    }

    pub fn get_default_install_dir() -> anyhow::Result<PathBuf> {
        let mut install_dir = dirs::data_local_dir().ok_or_else(|| anyhow::anyhow!("Failed to get local app data directory"))?;
        install_dir.push("Programs");
        install_dir.push("Squalr");
        Ok(install_dir)
    }
}
