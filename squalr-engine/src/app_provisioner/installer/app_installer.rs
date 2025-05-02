use crate::app_provisioner::app_provisioner_config::AppProvisionerConfig;
use crate::app_provisioner::installer::install_phase::InstallPhase;
use crate::app_provisioner::installer::install_progress::InstallProgress;
use crate::app_provisioner::operations::download::update_operation_download::UpdateOperationDownload;
use crate::app_provisioner::operations::extract::update_operation_extract::UpdateOperationExtract;
use crate::app_provisioner::operations::version_check::version_checker_status::VersionCheckerStatus;
use crate::app_provisioner::operations::version_check::version_checker_task::VersionCheckerTask;
use crate::app_provisioner::progress_tracker::ProgressTracker;
use squalr_engine_common::file_system::file_system_utils::FileSystemUtils;
use std::path::PathBuf;
use tempfile;

pub struct AppInstaller {}

impl AppInstaller {
    // let app_exe = self.install_dir.join("Squalr.exe");
    pub fn run_installation(
        install_dir: PathBuf,
        progress_tracker: ProgressTracker,
    ) {
        let progress_tracker = progress_tracker.clone();

        VersionCheckerTask::run(move |status| {
            if let VersionCheckerStatus::LatestVersionFound(latest_version) = status {
                log::info!("Starting installation...");

                // Create temporary directory for downloads.
                let tmp_dir = match tempfile::Builder::new().prefix("app").tempdir() {
                    Ok(dir) => dir,
                    Err(err) => {
                        log::error!("Failed to create temp directory: {err}");
                        return;
                    }
                };

                let tmp_file_path = tmp_dir.path().join(AppProvisionerConfig::FILENAME);
                log::info!("Temporary file location: {}", tmp_file_path.display());

                // Setup for downloading the new version.
                progress_tracker.init_progress();
                let download_url = AppProvisionerConfig::get_latest_version_url();
                let FIX_ME = 420; // Need to parse download version from version checker

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
                if let Err(err) = downloader.download_file(&download_url, &tmp_file_path) {
                    log::error!("Failed to download app: {err}");
                    return;
                }

                // Extract to a temporary location first.
                let tmp_extract_dir = tmp_dir.path().join("extracted");
                if let Err(err) = std::fs::create_dir_all(&tmp_extract_dir) {
                    log::error!("Failed to create temporary extraction directory: {err}");
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
                if let Err(err) = extractor.extract_archive(&tmp_file_path) {
                    log::error!("Failed to extract zip archive: {err}");
                    return;
                }

                // Regular installation - clear directory and copy new files.
                if let Err(err) = Self::replace_installation_directory_contents(&tmp_extract_dir, &install_dir) {
                    log::error!("Failed to update installation directory: {err}");
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
        log::info!("Copying new files from {} to {}", src_dir.display(), dst_dir.display());
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
