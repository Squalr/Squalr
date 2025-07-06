use crate::app_provisioner::app_provisioner_config::AppProvisionerConfig;
use crate::app_provisioner::installer::install_phase::InstallPhase;
use crate::app_provisioner::installer::install_progress::InstallProgress;
use crate::app_provisioner::operations::download::update_operation_download::UpdateOperationDownload;
use crate::app_provisioner::operations::extract::update_operation_extract::UpdateOperationExtract;
use crate::app_provisioner::operations::version_check::version_checker_status::VersionCheckerStatus;
use crate::app_provisioner::operations::version_check::version_checker_task::VersionCheckerTask;
use crate::app_provisioner::progress_tracker::ProgressTracker;
use squalr_engine_common::file_system::file_system_utils::FileSystemUtils;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
use tempfile;

pub struct AppUpdater {}

impl AppUpdater {
    pub fn run_update(progress_tracker: ProgressTracker) {
        let progress_tracker = progress_tracker.clone();
        let install_dir = match FileSystemUtils::get_executable_path().parent() {
            Some(path) => path.to_owned(),
            None => {
                log::warn!("Failed to resolve install diretory! Updater cannot run.");
                return;
            }
        };

        VersionCheckerTask::run(move |status| {
            if let VersionCheckerStatus::LatestVersionFound(latest_version_info) = status {
                let release_tag_version = latest_version_info.tag_name.trim_start_matches('v');
                let Ok(latest_version) = semver::Version::from_str(&release_tag_version) else {
                    log::error!("Failed to parse latest app version from latest version info, unable to check for app updates.");
                    return;
                };
                let Ok(current_version) = semver::Version::from_str(env!("CARGO_PKG_VERSION")) else {
                    log::error!("Failed to parse current app version, unable to check for app updates.");
                    return;
                };

                if latest_version <= current_version {
                    log::info!("Squalr is up to date, no updates available.");
                    return;
                }

                // Find the .zip asset meta data for the latest github release.
                let maybe_zip_asset = latest_version_info.assets.as_ref().and_then(|assets| {
                    assets
                        .iter()
                        .find(|asset| asset.name.eq_ignore_ascii_case("squalr.zip"))
                });
                let Some(zip_asset) = maybe_zip_asset else {
                    log::error!("Could not find squalr.zip in release assets, update failed.");
                    return;
                };
                let download_url = &zip_asset.browser_download_url;

                log::info!("Starting update...");

                // Check if we're updating ourselves.
                let current_exe = FileSystemUtils::get_executable_path();
                let target_exe = install_dir.join("squalr.exe");
                let is_self_update = current_exe == target_exe;

                if is_self_update {
                    log::info!(
                        "Detected self-update scenario. Current exe: {}, Target exe: {}",
                        current_exe.display(),
                        target_exe.display()
                    );
                }

                // Create temporary directory for downloads.
                let tmp_dir = match tempfile::Builder::new().prefix("squalr").tempdir() {
                    Ok(dir) => dir,
                    Err(error) => {
                        log::error!("Failed to create temp directory: {}", error);
                        return;
                    }
                };

                let tmp_file_path = tmp_dir.path().join(AppProvisionerConfig::FILENAME);
                log::info!("Temporary file location: {}", tmp_file_path.display());

                // Download new version.
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

                // Download the new version
                let downloader = UpdateOperationDownload::new(progress_tracker.get_progress().clone(), download_progress_callback);
                if let Err(error) = downloader.download_file(&download_url, &tmp_file_path) {
                    log::error!("Failed to download app: {}", error);
                    return;
                }

                // Extract to a temporary location first
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

                // Extract the archive
                let extractor = UpdateOperationExtract::new(&tmp_extract_dir, extract_progress_callback);
                if let Err(error) = extractor.extract_archive(&tmp_file_path) {
                    log::error!("Failed to extract zip archive: {}", error);
                    return;
                }

                // Perform self-update
                log::info!("Performing self-update...");
                let new_exe = tmp_extract_dir.join("squalr.exe");

                // Verify the new executable exists
                if !new_exe.exists() {
                    log::error!("New executable not found at expected path: {}", new_exe.display());
                    return;
                }

                // Try to clear and update auxiliary files.
                match Self::update_installation_directory(&tmp_extract_dir, &install_dir) {
                    Ok(_) => {
                        // Attempt self-replace
                        match self_replace::self_replace(&new_exe) {
                            Ok(_) => {
                                log::info!("Successfully replaced main executable!");
                            }
                            Err(error) => {
                                log::error!("Failed to replace main executable: {}", error);
                                return;
                            }
                        }
                    }
                    Err(error) => {
                        log::error!("Failed to update installation directory: {}", error);
                    }
                }

                // Update progress to complete
                progress_tracker.update_progress(InstallProgress {
                    phase: InstallPhase::Complete,
                    progress_percent: 1.0,
                    bytes_processed: 0,
                    total_bytes: 0,
                });

                log::info!("Update complete!");
            }
        });
    }

    fn update_installation_directory(
        source_directory: &Path,
        destination_directory: &Path,
    ) -> std::io::Result<()> {
        log::info!("Updating installation directory: {}", destination_directory.display());

        // Create the installation directory if it doesn't exist
        if !destination_directory.exists() {
            std::fs::create_dir_all(destination_directory)?;
        }

        // Clear existing files (best-effort, skip files that are in use).
        if destination_directory.exists() {
            log::info!("Clearing existing installation directory.");
            for entry in std::fs::read_dir(destination_directory)? {
                let entry = entry?;
                let path = entry.path();

                // Skip the main exe file as it should be in use (at least on Windows systems).
                if Self::is_main_exe(&path) {
                    log::info!("Deferring removing main executable at: {}", path.display());
                    continue;
                }

                if path.is_dir() {
                    match std::fs::remove_dir_all(&path) {
                        Ok(_) => log::info!("Removed directory: {}", path.display()),
                        Err(error) => log::warn!("Failed to remove directory {}: {}", path.display(), error),
                    }
                } else {
                    match std::fs::remove_file(&path) {
                        Ok(_) => log::info!("Removed file: {}", path.display()),
                        Err(error) => log::warn!("Failed to remove file {}: {}", path.display(), error),
                    }
                }
            }
        }

        // Copy new files.
        log::info!("Copying new files from {} to {}", source_directory.display(), destination_directory.display());

        for entry in std::fs::read_dir(source_directory)? {
            let entry = entry?;
            let source_file_path = entry.path();
            let destination_file_path = destination_directory.join(entry.file_name());

            if source_file_path.is_dir() {
                FileSystemUtils::copy_dir_all(&source_file_path, &destination_file_path)?;
            } else {
                if Self::is_main_exe(&destination_file_path) {
                    log::info!("Deferring copying main executable at: {}", destination_file_path.display());
                    continue;
                }

                match std::fs::copy(&source_file_path, &destination_file_path) {
                    Ok(_) => log::info!("Copied {} to {}", source_file_path.display(), destination_file_path.display()),
                    Err(error) => {
                        log::error!(
                            "Failed to copy {} to {}: {}",
                            source_file_path.display(),
                            destination_file_path.display(),
                            error
                        );
                        return Err(error);
                    }
                }
            }
        }

        Ok(())
    }

    fn is_main_exe(target_exe_path: &PathBuf) -> bool {
        let current_exe_path = match FileSystemUtils::get_executable_path().canonicalize() {
            Ok(path) => path,
            Err(error) => {
                log::warn!("Failed to resolve current executable path: {}", error);
                return false;
            }
        };

        let target_exe_path = match target_exe_path.canonicalize() {
            Ok(path) => path,
            Err(error) => {
                log::warn!("Failed to resolve target executable path, but the installation can still proceed: {}", error);
                return true;
            }
        };

        // Skip the file if it's the currently running executable.
        if !current_exe_path.as_os_str().is_empty() && current_exe_path == target_exe_path {
            return true;
        }

        false
    }
}
