use crate::app_provisioner::app_provisioner_config::AppProvisionerConfig;
use crate::app_provisioner::installer::install_phase::InstallPhase;
use crate::app_provisioner::installer::install_progress::InstallProgress;
use crate::app_provisioner::operations::download::update_operation_download::UpdateOperationDownload;
use crate::app_provisioner::operations::extract::update_operation_extract::UpdateOperationExtract;
use crate::app_provisioner::operations::version_check::version_checker_status::VersionCheckerStatus;
use crate::app_provisioner::operations::version_check::version_checker_task::VersionCheckerTask;
use crate::app_provisioner::progress_tracker::ProgressTracker;
use squalr_engine_api::utils::file_system::file_system_utils::FileSystemUtils;
use std::path::PathBuf;
use tempfile;

pub struct AppInstaller {}

impl AppInstaller {
    pub fn run_installation(
        install_dir: PathBuf,
        progress_tracker: ProgressTracker,
    ) {
        let progress_tracker = progress_tracker.clone();

        VersionCheckerTask::run(move |status| {
            if let VersionCheckerStatus::LatestVersionFound(latest_version_info) = status {
                log::info!("Starting installation...");

                // Find the .zip asset meta data for the latest github release.
                let maybe_zip_asset = latest_version_info.assets.as_ref().and_then(|assets| {
                    assets
                        .iter()
                        .find(|asset| asset.name.eq_ignore_ascii_case("squalr.zip"))
                });
                let Some(zip_asset) = maybe_zip_asset else {
                    log::error!("Could not find squalr.zip in release assets, installation failed.");
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

    fn replace_installation_directory_contents(
        src_dir: &std::path::Path,
        dst_dir: &std::path::Path,
    ) -> std::io::Result<()> {
        log::info!("Updating installation directory contents: {}", dst_dir.display());

        // Create the installation directory if it doesn't exist.
        if !dst_dir.exists() {
            std::fs::create_dir_all(dst_dir)?;
        }

        // Clear existing files.
        if dst_dir.exists() {
            log::info!("Clearing existing installation directory");
            for entry in std::fs::read_dir(dst_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    std::fs::remove_dir_all(&path)?;
                } else {
                    std::fs::remove_file(&path)?;
                }
            }
        }

        // Copy new files.
        log::info!("Copying new files... from");
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
