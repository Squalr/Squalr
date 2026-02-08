use crate::ui_state::InstallerUiState;
use eframe::egui::Context;
use squalr_engine::app_provisioner::app_provisioner_config::AppProvisionerConfig;
use squalr_engine::app_provisioner::installer::app_installer::AppInstaller;
use squalr_engine::app_provisioner::installer::install_phase::InstallPhase;
use squalr_engine::app_provisioner::operations::launch::update_operation_launch::UpdateOperationLaunch;
use squalr_engine::app_provisioner::progress_tracker::ProgressTracker;
use std::sync::{Arc, Mutex};
use std::thread;

pub(crate) fn launch_app() {
    match AppProvisionerConfig::get_default_install_dir() {
        Ok(app_install_directory) => {
            let executable_path = app_install_directory.join("squalr.exe");
            UpdateOperationLaunch::launch_app(&executable_path);
        }
        Err(error) => {
            log::error!("Failed to resolve install directory: {error}");
        }
    }
}

pub(crate) fn install_phase_string(install_phase: InstallPhase) -> &'static str {
    match install_phase {
        InstallPhase::Download => "Downloading installer package.",
        InstallPhase::Extraction => "Extracting installation payload.",
        InstallPhase::Complete => "Installation complete.",
    }
}

pub(crate) fn installer_status_string(ui_state: &InstallerUiState) -> &'static str {
    if ui_state.install_complete {
        "Squalr installed successfully."
    } else {
        "Installing Squalr, please wait..."
    }
}

pub(crate) fn start_installer(
    ui_state: Arc<Mutex<InstallerUiState>>,
    repaint_context: Context,
) {
    let progress_tracker = ProgressTracker::new();
    let progress_receiver = progress_tracker.subscribe();
    let progress_ui_state = ui_state.clone();
    let progress_repaint_context = repaint_context.clone();

    thread::spawn(move || {
        for install_progress in progress_receiver {
            if let Ok(mut state) = progress_ui_state.lock() {
                state.set_progress(install_progress);
                progress_repaint_context.request_repaint();
            }
        }
    });

    match AppProvisionerConfig::get_default_install_dir() {
        Ok(install_directory) => {
            AppInstaller::run_installation(install_directory, progress_tracker);
        }
        Err(error) => {
            log::error!("Failed to resolve install directory: {error}");
        }
    }
}
