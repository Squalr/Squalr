use crate::installer_runtime::start_installer;
use crate::theme::InstallerTheme;
use crate::ui_assets::{InstallerIconLibrary, load_installer_icon_library};
use crate::ui_state::InstallerUiState;
use crate::views::main_window::installer_main_window_view::InstallerMainWindowView;
use eframe::egui;
use eframe::egui::{Frame, Visuals};
use epaint::{CornerRadius, Rgba, vec2};
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
        start_installer(ui_state.clone(), context.clone());

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

        let state_snapshot = match self.ui_state.lock() {
            Ok(ui_state) => ui_state.clone(),
            Err(_) => InstallerUiState::new(),
        };

        let app_frame = Frame::new()
            .corner_radius(self.corner_radius)
            .stroke(context.style().visuals.widgets.noninteractive.fg_stroke)
            .outer_margin(2.0);

        egui::CentralPanel::default()
            .frame(app_frame)
            .show(context, |user_interface| {
                user_interface.style_mut().spacing.item_spacing = vec2(0.0, 0.0);

                let installer_main_window_view = InstallerMainWindowView::new(self.installer_theme.clone(), self.installer_icon_library.clone());
                installer_main_window_view.show(user_interface, &state_snapshot);
            });
    }
}
