use crate::logging::{MAX_LOG_BUFFER_BYTES, trim_log_buffer};
use squalr_engine::app_provisioner::app_provisioner_config::AppProvisionerConfig;
use squalr_engine::app_provisioner::installer::install_phase::InstallPhase;
use squalr_engine::app_provisioner::installer::install_progress::InstallProgress;
use squalr_engine::app_provisioner::installer::install_shortcut_options::InstallShortcutOptions;
use std::path::PathBuf;

#[derive(Clone)]
pub(crate) struct InstallerUiState {
    pub(crate) installer_phase: InstallPhase,
    pub(crate) installer_progress: f32,
    pub(crate) installer_progress_string: String,
    pub(crate) install_directory_input: String,
    pub(crate) install_started: bool,
    pub(crate) install_permission_granted: bool,
    pub(crate) install_configuration_error: Option<String>,
    pub(crate) install_complete: bool,
    pub(crate) install_shortcut_options: InstallShortcutOptions,
    pub(crate) installer_logs: String,
}

impl InstallerUiState {
    pub(crate) fn new() -> Self {
        let install_directory_input = AppProvisionerConfig::get_default_install_dir()
            .map(|default_install_directory| default_install_directory.to_string_lossy().into_owned())
            .unwrap_or_else(|_| String::new());

        Self {
            installer_phase: InstallPhase::Download,
            installer_progress: 0.0,
            installer_progress_string: "0%".to_string(),
            install_directory_input,
            install_started: false,
            install_permission_granted: false,
            install_configuration_error: None,
            install_complete: false,
            install_shortcut_options: InstallShortcutOptions::default(),
            installer_logs: String::new(),
        }
    }

    pub(crate) fn resolve_install_directory(&self) -> Result<PathBuf, String> {
        let install_directory_text = self.install_directory_input.trim();
        if install_directory_text.is_empty() {
            return Err("Installation directory is required.".to_string());
        }

        Ok(PathBuf::from(install_directory_text))
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
