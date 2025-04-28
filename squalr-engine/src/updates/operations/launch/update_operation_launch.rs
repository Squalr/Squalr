use crate::updates::progress_tracker::ProgressTracker;
use std::path::PathBuf;

pub struct AppInstaller {
    install_dir: PathBuf,
    progress_tracker: ProgressTracker,
}

impl AppInstaller {
    pub fn launch_app(app_executable_path: &PathBuf) {
        if !app_executable_path.exists() {
            log::error!("App executable not found at: {}", app_executable_path.display());
            return;
        }

        match std::process::Command::new(&app_executable_path).spawn() {
            Ok(_) => {
                log::info!("Successfully launched Squalr");
                std::process::exit(0);
            }
            Err(err) => {
                log::error!("Failed to launch Squalr: {err}");
            }
        }
    }
}
