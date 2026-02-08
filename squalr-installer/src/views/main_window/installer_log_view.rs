use crate::theme::InstallerTheme;
use crate::ui_state::InstallerUiState;
use eframe::egui::{Align, Color32, Frame, Layout, Margin, RichText, ScrollArea, Stroke, Ui};

#[derive(Clone)]
pub(crate) struct InstallerLogView {
    installer_theme: InstallerTheme,
}

impl InstallerLogView {
    pub(crate) fn new(installer_theme: InstallerTheme) -> Self {
        Self { installer_theme }
    }

    pub(crate) fn show(
        &self,
        user_interface: &mut Ui,
        installer_state: &InstallerUiState,
    ) {
        Frame::new()
            .fill(self.installer_theme.color_background_primary)
            .stroke(Stroke::new(1.0, self.installer_theme.color_border_panel))
            .inner_margin(Margin::same(8))
            .show(user_interface, |user_interface| {
                user_interface.horizontal(|user_interface| {
                    user_interface.label(
                        RichText::new("Installer Log")
                            .font(self.installer_theme.fonts.font_window_title.clone())
                            .color(self.installer_theme.color_foreground),
                    );

                    if installer_state.install_complete {
                        user_interface.with_layout(Layout::right_to_left(Align::Center), |user_interface| {
                            user_interface.label(
                                RichText::new("Ready")
                                    .font(self.installer_theme.fonts.font_normal.clone())
                                    .color(self.installer_theme.color_background_control_success_dark),
                            );
                        });
                    }
                });

                Frame::new()
                    .fill(self.installer_theme.color_log_background)
                    .stroke(Stroke::new(1.0, self.installer_theme.color_border_panel))
                    .inner_margin(Margin::same(8))
                    .show(user_interface, |user_interface| {
                        user_interface.set_min_height(290.0);
                        user_interface.set_min_width(user_interface.available_width());

                        ScrollArea::vertical()
                            .id_salt("installer_log_scroll")
                            .auto_shrink([false, false])
                            .stick_to_bottom(true)
                            .show(user_interface, |user_interface| {
                                if installer_state.installer_logs.is_empty() {
                                    user_interface.label(
                                        RichText::new("Waiting for installer output.")
                                            .font(self.installer_theme.fonts.font_ubuntu_mono_normal.clone())
                                            .color(self.installer_theme.color_foreground_preview),
                                    );
                                } else {
                                    for installer_log_line in installer_state.installer_logs.lines() {
                                        user_interface.label(
                                            RichText::new(installer_log_line)
                                                .font(self.installer_theme.fonts.font_ubuntu_mono_normal.clone())
                                                .color(log_color_for_line(installer_log_line, &self.installer_theme)),
                                        );
                                    }
                                }
                            });
                    });
            });
    }
}

fn log_color_for_line(
    log_line: &str,
    installer_theme: &InstallerTheme,
) -> Color32 {
    if log_line.starts_with("[ERROR]") {
        installer_theme.color_foreground_error
    } else if log_line.starts_with("[WARN]") {
        installer_theme.color_foreground_warning
    } else if log_line.starts_with("[DEBUG]") {
        installer_theme.color_foreground_debug
    } else if log_line.starts_with("[TRACE]") {
        installer_theme.color_foreground_trace
    } else if log_line.starts_with("[INFO]") {
        installer_theme.color_foreground_info
    } else {
        installer_theme.color_foreground
    }
}
