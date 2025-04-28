use crate::updates::downloader::file_downloader::FileDownloader;
use crate::updates::extractor::app_extractor::AppExtractor;
use crate::updates::shared::app_download_endpoints::AppDownloadEndpoints;
use crate::updates::shared::install_phase::InstallPhase;
use crate::updates::shared::install_progress::InstallProgress;
use crate::updates::shared::progress_tracker::ProgressTracker;
use anyhow::Result;
use squalr_engine_common::file_system::file_system_utils::FileSystemUtils;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Once, RwLock};
use tempfile;

pub struct AppUpdater {
    install_dir: PathBuf,
    progress_tracker: ProgressTracker,
}

impl AppUpdater {
    fn new() -> Self {
        let install_dir = match Self::get_default_install_dir() {
            Ok(dir) => {
                log::info!("Install directory: {}", dir.display());
                dir
            }
            Err(err) => {
                log::error!("Failed to get install directory: {err}");
                PathBuf::from("")
            }
        };

        Self {
            install_dir,
            progress_tracker: ProgressTracker::new(InstallPhase::Download),
        }
    }

    pub fn get_instance() -> Arc<RwLock<AppUpdater>> {
        static mut INSTANCE: Option<Arc<RwLock<AppUpdater>>> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = Arc::new(RwLock::new(AppUpdater::new()));
                INSTANCE = Some(instance);
            });

            #[allow(static_mut_refs)]
            return INSTANCE.as_ref().unwrap_unchecked().clone();
        }
    }

    pub fn launch_app(&self) {
        let app_exe = self.install_dir.join("app.exe");

        if !app_exe.exists() {
            log::error!("App executable not found at: {}", app_exe.display());
            return;
        }

        match std::process::Command::new(&app_exe).spawn() {
            Ok(_) => {
                log::info!("Successfully launched app.");
                std::process::exit(0);
            }
            Err(err) => {
                log::error!("Failed to launch app: {err}");
            }
        }
    }

    pub fn begin_update(&self) {
        Self::run_update();
    }

    pub fn subscribe(&self) -> Receiver<InstallProgress> {
        self.progress_tracker.subscribe()
    }

    fn run_update() {
        let app_instance = AppUpdater::get_instance();
        let app_updater = match app_instance.read() {
            Ok(updater) => updater,
            Err(err) => {
                log::error!("Error accessing app updater: {err}");
                return;
            }
        };

        log::info!("Starting update...");

        // Check if we're updating ourselves.
        let current_exe = FileSystemUtils::get_executable_path();
        let target_exe = app_updater.install_dir.join("app.exe");
        let is_self_update = current_exe == target_exe;

        if is_self_update {
            log::info!(
                "Detected self-update scenario. Current exe: {}, Target exe: {}",
                current_exe.display(),
                target_exe.display()
            );
        }

        // Create temporary directory for downloads.
        let tmp_dir = match tempfile::Builder::new().prefix("app").tempdir() {
            Ok(dir) => dir,
            Err(err) => {
                log::error!("Failed to create temp directory: {err}");
                return;
            }
        };

        let tmp_file_path = tmp_dir.path().join(AppDownloadEndpoints::FILENAME);
        log::info!("Temporary file location: {}", tmp_file_path.display());

        // Download new version.
        app_updater.progress_tracker.init_progress();
        let download_url = AppDownloadEndpoints::get_latest_version_url();
        let FIX_ME = 420; // Need to parse download version from version checker

        // Download progress callback setup.
        let instance = app_instance.clone();
        let download_progress_callback = Box::new(move |bytes_downloaded: u64, total_bytes: u64| {
            if let Ok(updater) = instance.read() {
                let progress = InstallProgress {
                    phase: InstallPhase::Download,
                    progress_percent: (bytes_downloaded as f32 / total_bytes as f32) * AppDownloadEndpoints::DOWNLOAD_WEIGHT,
                    bytes_processed: bytes_downloaded,
                    total_bytes,
                };
                updater.progress_tracker.update_progress(progress);
            }
        });

        // Download the new version
        let downloader = FileDownloader::new(app_updater.progress_tracker.get_progress().clone(), download_progress_callback);
        if let Err(err) = downloader.download_file(&download_url, &tmp_file_path) {
            log::error!("Failed to download app: {err}");
            return;
        }

        // Extract to a temporary location first
        let tmp_extract_dir = tmp_dir.path().join("extracted");
        if let Err(err) = std::fs::create_dir_all(&tmp_extract_dir) {
            log::error!("Failed to create temporary extraction directory: {err}");
            return;
        }

        // Extract progress callback setup
        let instance = app_instance.clone();
        let extract_progress_callback = Box::new(move |files_processed: u64, total_files: u64| {
            if let Ok(updater) = instance.read() {
                let progress = InstallProgress {
                    phase: InstallPhase::Extraction,
                    progress_percent: AppDownloadEndpoints::DOWNLOAD_WEIGHT
                        + (files_processed as f32 / total_files as f32) * AppDownloadEndpoints::EXTRACT_WEIGHT,
                    bytes_processed: files_processed,
                    total_bytes: total_files,
                };
                updater.progress_tracker.update_progress(progress);
            }
        });

        // Extract the archive
        let extractor = AppExtractor::new(&tmp_extract_dir, extract_progress_callback);
        if let Err(err) = extractor.extract_archive(&tmp_file_path) {
            log::error!("Failed to extract zip archive: {err}");
            return;
        }

        // Perform self-update
        log::info!("Performing self-update...");
        let new_exe = tmp_extract_dir.join("app.exe");

        // Verify the new executable exists
        if !new_exe.exists() {
            log::error!("New executable not found at expected path: {}", new_exe.display());
            return;
        }

        // Attempt self-replace
        match self_replace::self_replace(&new_exe) {
            Ok(_) => {
                log::info!("Successfully replaced executable");

                // After successful self-replace, try to clear and update other files
                if let Err(err) = Self::update_installation_directory(&tmp_extract_dir, &app_updater.install_dir) {
                    log::error!("Failed to update installation directory after self-replace: {err}");
                    // Continue anyway since the exe was replaced successfully
                }
            }
            Err(err) => {
                log::error!("Failed to perform self-replace: {err}");
                return;
            }
        }

        // Update progress to complete
        app_updater.progress_tracker.update_progress(InstallProgress {
            phase: InstallPhase::Complete,
            progress_percent: 1.0,
            bytes_processed: 0,
            total_bytes: 0,
        });

        log::info!("Update complete!");
    }

    pub fn get_install_dir(&self) -> &PathBuf {
        &self.install_dir
    }

    fn get_default_install_dir() -> Result<PathBuf> {
        let mut install_dir = dirs::data_local_dir().ok_or_else(|| anyhow::anyhow!("Failed to get local app data directory"))?;
        install_dir.push("Programs");
        install_dir.push("Squalr");
        Ok(install_dir)
    }

    fn update_installation_directory(
        src_dir: &std::path::Path,
        dst_dir: &std::path::Path,
    ) -> std::io::Result<()> {
        log::info!("Updating installation directory: {}", dst_dir.display());

        // Create the installation directory if it doesn't exist
        if !dst_dir.exists() {
            std::fs::create_dir_all(dst_dir)?;
        }

        // Clear existing files (best-effort, skip files that are in use)
        if dst_dir.exists() {
            log::info!("Clearing existing installation directory");
            for entry in std::fs::read_dir(dst_dir)? {
                let entry = entry?;
                let path = entry.path();

                // Skip the exe file as it might be in use
                if path.extension().and_then(|ext| ext.to_str()) == Some("exe") {
                    log::info!("Skipping executable: {}", path.display());
                    continue;
                }

                if path.is_dir() {
                    match std::fs::remove_dir_all(&path) {
                        Ok(_) => log::info!("Removed directory: {}", path.display()),
                        Err(e) => log::warn!("Failed to remove directory {}: {}", path.display(), e),
                    }
                } else {
                    match std::fs::remove_file(&path) {
                        Ok(_) => log::info!("Removed file: {}", path.display()),
                        Err(e) => log::warn!("Failed to remove file {}: {}", path.display(), e),
                    }
                }
            }
        }

        // Copy new files
        log::info!("Copying new files from {} to {}", src_dir.display(), dst_dir.display());
        for entry in std::fs::read_dir(src_dir)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst_dir.join(entry.file_name());

            if src_path.is_dir() {
                FileSystemUtils::copy_dir_all(&src_path, &dst_path)?;
            } else {
                // Skip copying the exe in self-update scenarios as it's handled by self_replace
                if dst_path.extension().and_then(|ext| ext.to_str()) == Some("exe") && dst_path.exists() {
                    log::info!("Skipping copying exe as it will be handled by self_replace: {}", dst_path.display());
                    continue;
                }

                match std::fs::copy(&src_path, &dst_path) {
                    Ok(_) => log::info!("Copied {} to {}", src_path.display(), dst_path.display()),
                    Err(e) => {
                        log::error!("Failed to copy {} to {}: {}", src_path.display(), dst_path.display(), e);
                        return Err(e);
                    }
                }
            }
        }

        Ok(())
    }
}
