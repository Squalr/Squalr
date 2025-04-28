use crate::updates::downloader::app_downloader::AppDownloader;
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

pub struct AppInstaller {
    install_dir: PathBuf,
    progress_tracker: ProgressTracker,
}

impl AppInstaller {
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

    pub fn get_instance() -> Arc<RwLock<AppInstaller>> {
        static mut INSTANCE: Option<Arc<RwLock<AppInstaller>>> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = Arc::new(RwLock::new(AppInstaller::new()));
                INSTANCE = Some(instance);
            });

            #[allow(static_mut_refs)]
            return INSTANCE.as_ref().unwrap_unchecked().clone();
        }
    }

    pub fn launch_app(&self) {
        let app_exe = self.install_dir.join("Squalr.exe");

        if !app_exe.exists() {
            log::error!("App executable not found at: {}", app_exe.display());
            return;
        }

        match std::process::Command::new(&app_exe).spawn() {
            Ok(_) => {
                log::info!("Successfully launched Squalr");
                std::process::exit(0);
            }
            Err(err) => {
                log::error!("Failed to launch Squalr: {err}");
            }
        }
    }

    pub fn begin_install(&self) {
        Self::run_installation();
    }

    pub fn subscribe(&self) -> Receiver<InstallProgress> {
        self.progress_tracker.subscribe()
    }

    fn run_installation() {
        let app_instance = AppInstaller::get_instance();
        let app_installer = match app_instance.read() {
            Ok(installer) => installer,
            Err(err) => {
                log::error!("Error accessing app installer: {err}");
                return;
            }
        };

        log::info!("Starting installation...");

        // Create temporary directory for downloads
        let tmp_dir = match tempfile::Builder::new().prefix("app").tempdir() {
            Ok(dir) => dir,
            Err(err) => {
                log::error!("Failed to create temp directory: {err}");
                return;
            }
        };

        let tmp_file_path = tmp_dir.path().join(AppDownloadEndpoints::FILENAME);
        log::info!("Temporary file location: {}", tmp_file_path.display());

        // Download new version
        app_installer.progress_tracker.init_progress();
        let download_url = AppDownloadEndpoints::get_latest_download_url();

        // Download progress callback setup
        let instance = app_instance.clone();
        let download_progress_callback = Box::new(move |bytes_downloaded: u64, total_bytes: u64| {
            if let Ok(installer) = instance.read() {
                let progress = InstallProgress {
                    phase: InstallPhase::Download,
                    progress_percent: (bytes_downloaded as f32 / total_bytes as f32) * AppDownloadEndpoints::DOWNLOAD_WEIGHT,
                    bytes_processed: bytes_downloaded,
                    total_bytes,
                };
                installer.progress_tracker.update_progress(progress);
            }
        });

        // Download the new version
        let downloader = AppDownloader::new(app_installer.progress_tracker.get_progress().clone(), download_progress_callback);
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
            if let Ok(installer) = instance.read() {
                let progress = InstallProgress {
                    phase: InstallPhase::Extraction,
                    progress_percent: AppDownloadEndpoints::DOWNLOAD_WEIGHT
                        + (files_processed as f32 / total_files as f32) * AppDownloadEndpoints::EXTRACT_WEIGHT,
                    bytes_processed: files_processed,
                    total_bytes: total_files,
                };
                installer.progress_tracker.update_progress(progress);
            }
        });

        // Extract the archive
        let extractor = AppExtractor::new(&tmp_extract_dir, extract_progress_callback);
        if let Err(err) = extractor.extract_archive(&tmp_file_path) {
            log::error!("Failed to extract zip archive: {err}");
            return;
        }

        // Regular installation - clear directory and copy new files
        if let Err(err) = Self::update_installation_directory(&tmp_extract_dir, &app_installer.install_dir) {
            log::error!("Failed to update installation directory: {err}");
            return;
        }

        // Update progress to complete
        app_installer.progress_tracker.update_progress(InstallProgress {
            phase: InstallPhase::Complete,
            progress_percent: 1.0,
            bytes_processed: 0,
            total_bytes: 0,
        });

        log::info!("Installation complete!");
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

        // Clear existing files
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

        // Copy new files
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
