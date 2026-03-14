use crate::installer_runtime::start_installer;
use crate::theme::InstallerTheme;
use crate::ui_assets::{InstallerIconLibrary, load_installer_icon_library};
use crate::ui_state::InstallerUiState;
use crate::views::main_window::installer_main_window_view::InstallerMainWindowView;
use eframe::egui;
use eframe::egui::{Frame, Visuals};
use epaint::{CornerRadius, Rgba, vec2};
use squalr_engine::app_provisioner::installer::install_shortcut_options::InstallShortcutOptions;
use std::sync::{Arc, Mutex};

pub(crate) struct InstallerApp {
    ui_state: Arc<Mutex<InstallerUiState>>,
    installer_theme: InstallerTheme,
    installer_icon_library: Option<InstallerIconLibrary>,
    corner_radius: CornerRadius,
}

impl InstallerApp {
    pub(crate) fn new(
        context: &egui::Context,
        ui_state: Arc<Mutex<InstallerUiState>>,
    ) -> Self {
        let installer_theme = InstallerTheme::default();
        installer_theme.apply(context);
        installer_theme.install_fonts(context);
        let corner_radius = CornerRadius::same(installer_theme.corner_radius_panel);

        let installer_icon_library = load_installer_icon_library(context);

        Self {
            ui_state,
            installer_theme,
            installer_icon_library,
            corner_radius,
        }
    }
}

impl eframe::App for InstallerApp {
    fn clear_color(
        &self,
        _visuals: &Visuals,
    ) -> [f32; 4] {
        Rgba::TRANSPARENT.to_array()
    }

    fn update(
        &mut self,
        context: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        // Reapply visuals each frame so native theme updates cannot restore default light panel colors.
        self.installer_theme.apply(context);

        let mut pending_install_request: Option<(std::path::PathBuf, InstallShortcutOptions)> = None;
        if let Ok(mut installer_state) = self.ui_state.lock() {
            if installer_state.install_permission_granted && !installer_state.install_started {
                match installer_state.resolve_install_directory() {
                    Ok(install_directory) => {
                        installer_state.install_started = true;
                        installer_state.install_configuration_error = None;
                        pending_install_request = Some((install_directory, installer_state.install_shortcut_options.clone()));
                    }
                    Err(error_message) => {
                        installer_state.install_permission_granted = false;
                        installer_state.install_configuration_error = Some(error_message);
                    }
                }
            }
        }

        if let Some((pending_install_directory, install_shortcut_options)) = pending_install_request {
            start_installer(self.ui_state.clone(), context.clone(), pending_install_directory, install_shortcut_options);
        }

        let app_frame = Frame::new()
            .corner_radius(self.corner_radius)
            .stroke(context.style().visuals.widgets.noninteractive.fg_stroke)
            .outer_margin(2.0);

        egui::CentralPanel::default()
            .frame(app_frame)
            .show(context, |user_interface| {
                user_interface.style_mut().spacing.item_spacing = vec2(0.0, 0.0);

                let installer_main_window_view = InstallerMainWindowView::new(self.installer_theme.clone(), self.installer_icon_library.clone());
                if let Ok(mut installer_state) = self.ui_state.lock() {
                    installer_main_window_view.show(user_interface, &mut installer_state);
                } else {
                    let mut installer_state = InstallerUiState::new();
                    installer_main_window_view.show(user_interface, &mut installer_state);
                }
            });
    }
}
