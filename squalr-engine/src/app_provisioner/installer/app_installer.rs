use crate::app_provisioner::app_provisioner_config::AppProvisionerConfig;
use crate::app_provisioner::installer::install_phase::InstallPhase;
use crate::app_provisioner::installer::install_progress::InstallProgress;
use crate::app_provisioner::operations::download::update_operation_download::UpdateOperationDownload;
use crate::app_provisioner::operations::extract::update_operation_extract::UpdateOperationExtract;
use crate::app_provisioner::operations::version_check::version_checker_status::VersionCheckerStatus;
use crate::app_provisioner::operations::version_check::version_checker_task::VersionCheckerTask;
use crate::app_provisioner::progress_tracker::ProgressTracker;
use squalr_engine_api::utils::file_system::file_system_utils::FileSystemUtils;
use std::path::{Path, PathBuf};
use tempfile;

pub struct AppInstaller {}

impl AppInstaller {
    const LOCAL_BINARY_BASE_NAMES: [&'static str; 3] = ["squalr", "squalr-cli", "squalr-tui"];

    pub fn run_installation(
        install_dir: PathBuf,
        progress_tracker: ProgressTracker,
    ) {
        match Self::install_from_local_payload(&install_dir, &progress_tracker) {
            Ok(true) => return,
            Ok(false) => {
                log::info!("Local installer payload not found. Falling back to latest GitHub release download.");
            }
            Err(error) => {
                log::error!("Failed local installation from installer payload: {}", error);
                return;
            }
        }

        let progress_tracker = progress_tracker.clone();

        VersionCheckerTask::run(move |status| {
            if let VersionCheckerStatus::LatestVersionFound(latest_version_info) = status {
                log::info!("Starting installation...");

                // Find the .zip asset meta data for the latest github release.
                let maybe_zip_asset = latest_version_info.assets.as_ref().and_then(|assets| {
                    if let Some(expected_bundle_asset_name) = AppProvisionerConfig::get_release_bundle_asset_name(&latest_version_info.tag_name) {
                        if let Some(bundle_asset) = assets.iter().find(|release_asset| {
                            release_asset
                                .name
                                .eq_ignore_ascii_case(&expected_bundle_asset_name)
                        }) {
                            return Some(bundle_asset);
                        }

                        log::warn!(
                            "Could not find platform bundle asset {} in release; trying legacy fallback.",
                            expected_bundle_asset_name
                        );
                    }

                    assets
                        .iter()
                        .find(|release_asset| release_asset.name.eq_ignore_ascii_case("squalr.zip"))
                });
                let Some(zip_asset) = maybe_zip_asset else {
                    log::error!("Could not find a compatible zip asset in release assets, installation failed.");
                    return;
                };
                let download_url = &zip_asset.browser_download_url;

                // Create temporary directory for downloads.
                let tmp_dir = match tempfile::Builder::new().prefix("app").tempdir() {
                    Ok(dir) => dir,
                    Err(error) => {
                        log::error!("Failed to create temp directory: {}", error);
                        return;
                    }
                };

                let tmp_file_path = tmp_dir.path().join(AppProvisionerConfig::FILENAME);
                log::info!("Temporary file location: {}", tmp_file_path.display());

                // Setup for downloading the new version.
                progress_tracker.init_progress();

                // Download progress callback setup.
                let progress_tracker_clone = progress_tracker.clone();
                let download_progress_callback = Box::new(move |bytes_downloaded: u64, total_bytes: u64| {
                    let progress = InstallProgress {
                        phase: InstallPhase::Download,
                        progress_percent: (bytes_downloaded as f32 / total_bytes as f32) * AppProvisionerConfig::DOWNLOAD_WEIGHT,
                        bytes_processed: bytes_downloaded,
                        total_bytes,
                    };
                    progress_tracker_clone.update_progress(progress);
                });

                // Download the new version.
                let downloader = UpdateOperationDownload::new(progress_tracker.get_progress().clone(), download_progress_callback);
                if let Err(error) = downloader.download_file(&download_url, &tmp_file_path) {
                    log::error!("Failed to download app: {}", error);
                    return;
                }

                // Extract to a temporary location first.
                let tmp_extract_dir = tmp_dir.path().join("extracted");
                if let Err(error) = std::fs::create_dir_all(&tmp_extract_dir) {
                    log::error!("Failed to create temporary extraction directory: {}", error);
                    return;
                }

                // Extract progress callback setup
                let progress_tracker_clone = progress_tracker.clone();
                let extract_progress_callback = Box::new(move |files_processed: u64, total_files: u64| {
                    let progress = InstallProgress {
                        phase: InstallPhase::Extraction,
                        progress_percent: AppProvisionerConfig::DOWNLOAD_WEIGHT
                            + (files_processed as f32 / total_files as f32) * AppProvisionerConfig::EXTRACT_WEIGHT,
                        bytes_processed: files_processed,
                        total_bytes: total_files,
                    };
                    progress_tracker_clone.update_progress(progress);
                });

                // Extract the archive.
                let extractor = UpdateOperationExtract::new(&tmp_extract_dir, extract_progress_callback);
                if let Err(error) = extractor.extract_archive(&tmp_file_path) {
                    log::error!("Failed to extract zip archive: {}", error);
                    return;
                }

                // Regular installation - clear directory and copy new files.
                if let Err(error) = Self::replace_installation_directory_contents(&tmp_extract_dir, &install_dir) {
                    log::error!("Failed to update installation directory: {}", error);
                    return;
                }

                // Update progress to complete.
                progress_tracker.update_progress(InstallProgress {
                    phase: InstallPhase::Complete,
                    progress_percent: 1.0,
                    bytes_processed: 0,
                    total_bytes: 0,
                });

                log::info!("Installation complete!");
            }
        });
    }

    fn install_from_local_payload(
        install_dir: &Path,
        progress_tracker: &ProgressTracker,
    ) -> std::io::Result<bool> {
        let payload_directory = match Self::resolve_payload_directory() {
            Some(payload_directory) => payload_directory,
            None => return Ok(false),
        };

        let local_binary_paths = Self::resolve_local_binary_paths(&payload_directory);
        if local_binary_paths.is_empty() {
            return Ok(false);
        }

        if local_binary_paths.len() != Self::LOCAL_BINARY_BASE_NAMES.len() {
            log::warn!(
                "Incomplete local installer payload in {}. Expected {} binaries but found {}. Falling back to download flow.",
                payload_directory.display(),
                Self::LOCAL_BINARY_BASE_NAMES.len(),
                local_binary_paths.len()
            );
            return Ok(false);
        }

        log::info!("Installing from local payload in {}", payload_directory.display());
        Self::move_payload_files_into_installation_directory(&local_binary_paths, install_dir, progress_tracker)?;
        progress_tracker.update_progress(InstallProgress {
            phase: InstallPhase::Complete,
            progress_percent: 1.0,
            bytes_processed: 0,
            total_bytes: 0,
        });
        log::info!("Installation complete from local payload.");
        Ok(true)
    }

    fn resolve_payload_directory() -> Option<PathBuf> {
        std::env::current_exe()
            .ok()
            .and_then(|installer_executable_path| {
                installer_executable_path
                    .parent()
                    .map(|parent_path| parent_path.to_path_buf())
            })
    }

    fn resolve_local_binary_paths(payload_directory: &Path) -> Vec<PathBuf> {
        Self::LOCAL_BINARY_BASE_NAMES
            .iter()
            .filter_map(|binary_base_name| Self::resolve_local_binary_path(payload_directory, binary_base_name))
            .collect()
    }

    fn resolve_local_binary_path(
        payload_directory: &Path,
        binary_base_name: &str,
    ) -> Option<PathBuf> {
        let mut binary_name_candidates = vec![binary_base_name.to_string()];
        if !binary_base_name.ends_with(".exe") {
            binary_name_candidates.push(format!("{binary_base_name}.exe"));
        }

        binary_name_candidates
            .iter()
            .map(|binary_candidate_name| payload_directory.join(binary_candidate_name))
            .find(|binary_candidate_path| binary_candidate_path.is_file())
    }

    fn move_payload_files_into_installation_directory(
        source_file_paths: &[PathBuf],
        install_dir: &Path,
        progress_tracker: &ProgressTracker,
    ) -> std::io::Result<()> {
        Self::prepare_install_directory(install_dir)?;

        let total_file_count = source_file_paths.len() as u64;
        for (file_index, source_file_path) in source_file_paths.iter().enumerate() {
            let destination_file_name = source_file_path
                .file_name()
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Installer payload file had no file name"))?;
            let destination_file_path = install_dir.join(destination_file_name);

            Self::move_file_with_copy_fallback(source_file_path, &destination_file_path)?;

            progress_tracker.update_progress(InstallProgress {
                phase: InstallPhase::Extraction,
                progress_percent: (file_index as f32 + 1.0) / total_file_count as f32,
                bytes_processed: file_index as u64 + 1,
                total_bytes: total_file_count,
            });
        }

        Ok(())
    }

    fn move_file_with_copy_fallback(
        source_file_path: &Path,
        destination_file_path: &Path,
    ) -> std::io::Result<()> {
        match std::fs::rename(source_file_path, destination_file_path) {
            Ok(()) => Ok(()),
            Err(rename_error) => {
                if destination_file_path.exists() {
                    std::fs::remove_file(destination_file_path)?;
                }

                std::fs::copy(source_file_path, destination_file_path)?;
                std::fs::remove_file(source_file_path)?;

                log::warn!(
                    "Fell back to copy+delete while moving {} to {} due to rename failure: {}",
                    source_file_path.display(),
                    destination_file_path.display(),
                    rename_error
                );

                Ok(())
            }
        }
    }

    fn prepare_install_directory(install_dir: &Path) -> std::io::Result<()> {
        log::info!("Updating installation directory contents: {}", install_dir.display());

        if !install_dir.exists() {
            std::fs::create_dir_all(install_dir)?;
        }

        log::info!("Clearing existing installation directory.");
        for install_entry in std::fs::read_dir(install_dir)? {
            let install_entry = install_entry?;
            let install_entry_path = install_entry.path();

            if install_entry_path.is_dir() {
                std::fs::remove_dir_all(&install_entry_path)?;
            } else {
                std::fs::remove_file(&install_entry_path)?;
            }
        }

        Ok(())
    }

    fn replace_installation_directory_contents(
        src_dir: &std::path::Path,
        dst_dir: &std::path::Path,
    ) -> std::io::Result<()> {
        Self::prepare_install_directory(dst_dir)?;

        // Copy new files.
        log::info!("Copying new files from:");
        log::info!("Source: {}", src_dir.display());
        log::info!("Destination: {}", dst_dir.display());

        for entry in std::fs::read_dir(src_dir)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst_dir.join(entry.file_name());

            if src_path.is_dir() {
                FileSystemUtils::copy_dir_all(&src_path, &dst_path)?;
            } else {
                std::fs::copy(&src_path, &dst_path)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::AppInstaller;
    use crate::app_provisioner::progress_tracker::ProgressTracker;
    use std::path::Path;
    use tempfile::TempDir;

    fn create_payload_file(
        payload_directory: &Path,
        file_name: &str,
    ) -> std::io::Result<()> {
        let file_path = payload_directory.join(file_name);
        std::fs::write(file_path, b"payload")
    }

    #[test]
    fn resolves_three_local_payload_binaries_with_exe_suffix() -> std::io::Result<()> {
        let payload_temp_dir = TempDir::new()?;
        create_payload_file(payload_temp_dir.path(), "squalr.exe")?;
        create_payload_file(payload_temp_dir.path(), "squalr-cli.exe")?;
        create_payload_file(payload_temp_dir.path(), "squalr-tui.exe")?;

        let resolved_binary_paths = AppInstaller::resolve_local_binary_paths(payload_temp_dir.path());

        assert_eq!(resolved_binary_paths.len(), 3);
        assert!(
            resolved_binary_paths
                .iter()
                .all(|resolved_binary_path| resolved_binary_path.exists())
        );
        Ok(())
    }

    #[test]
    fn move_payload_files_clears_destination_and_removes_source() -> std::io::Result<()> {
        let source_temp_dir = TempDir::new()?;
        create_payload_file(source_temp_dir.path(), "squalr.exe")?;
        create_payload_file(source_temp_dir.path(), "squalr-cli.exe")?;
        create_payload_file(source_temp_dir.path(), "squalr-tui.exe")?;

        let destination_temp_dir = TempDir::new()?;
        std::fs::write(destination_temp_dir.path().join("stale.txt"), b"stale")?;

        let progress_tracker = ProgressTracker::new();
        let source_binary_paths = AppInstaller::resolve_local_binary_paths(source_temp_dir.path());
        AppInstaller::move_payload_files_into_installation_directory(&source_binary_paths, destination_temp_dir.path(), &progress_tracker)?;

        assert!(!destination_temp_dir.path().join("stale.txt").exists());
        assert!(destination_temp_dir.path().join("squalr.exe").exists());
        assert!(destination_temp_dir.path().join("squalr-cli.exe").exists());
        assert!(destination_temp_dir.path().join("squalr-tui.exe").exists());
        assert!(
            source_binary_paths
                .iter()
                .all(|source_binary_path| !source_binary_path.exists())
        );
        Ok(())
    }
}
