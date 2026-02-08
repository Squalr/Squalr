use crate::logging::{MAX_LOG_BUFFER_BYTES, trim_log_buffer};
use squalr_engine::app_provisioner::installer::install_phase::InstallPhase;
use squalr_engine::app_provisioner::installer::install_progress::InstallProgress;

#[derive(Clone)]
pub(crate) struct InstallerUiState {
    pub(crate) installer_phase: InstallPhase,
    pub(crate) installer_progress: f32,
    pub(crate) installer_progress_string: String,
    pub(crate) install_complete: bool,
    pub(crate) installer_logs: String,
}

impl InstallerUiState {
    pub(crate) fn new() -> Self {
        Self {
            installer_phase: InstallPhase::Download,
            installer_progress: 0.0,
            installer_progress_string: "0%".to_string(),
            install_complete: false,
            installer_logs: String::new(),
        }
    }

    pub(crate) fn set_progress(
        &mut self,
        install_progress: InstallProgress,
    ) {
        self.installer_phase = install_progress.phase;
        self.installer_progress = install_progress.progress_percent.clamp(0.0, 1.0);
        self.installer_progress_string = format!("{:.0}%", self.installer_progress * 100.0);
        self.install_complete = install_progress.phase == InstallPhase::Complete;
    }

    pub(crate) fn append_log(
        &mut self,
        log_message: &str,
    ) {
        self.installer_logs.push_str(log_message);
        trim_log_buffer(&mut self.installer_logs, MAX_LOG_BUFFER_BYTES);
    }
}
