use crate::installer_runtime::{install_phase_string, installer_status_string};
use crate::theme::InstallerTheme;
use crate::ui_assets::InstallerIconLibrary;
use crate::ui_state::InstallerUiState;
use crate::views::main_window::installer_footer_view::InstallerFooterView;
use crate::views::main_window::installer_log_view::InstallerLogView;
use crate::views::main_window::installer_title_bar_view::InstallerTitleBarView;
use eframe::egui::{Align, Frame, Layout, Margin, RichText, Stroke, Ui};

#[derive(Clone)]
pub(crate) struct InstallerMainWindowView {
    installer_theme: InstallerTheme,
    installer_icon_library: Option<InstallerIconLibrary>,
}

impl InstallerMainWindowView {
    pub(crate) fn new(
        installer_theme: InstallerTheme,
        installer_icon_library: Option<InstallerIconLibrary>,
    ) -> Self {
        Self {
            installer_theme,
            installer_icon_library,
        }
    }

    pub(crate) fn show(
        &self,
        user_interface: &mut Ui,
        installer_state: &InstallerUiState,
    ) {
        let previous_item_spacing = user_interface.style().spacing.item_spacing;
        user_interface.style_mut().spacing.item_spacing = eframe::egui::vec2(0.0, 0.0);

        let installer_footer_view = InstallerFooterView::new(self.installer_theme.clone(), installer_state.install_complete);
        let installer_footer_height = installer_footer_view.get_height();

        user_interface.add(InstallerTitleBarView::new(self.installer_theme.clone(), self.installer_icon_library.clone()));

        let content_height = (user_interface.available_height() - installer_footer_height).max(0.0);
        user_interface.allocate_ui_with_layout(
            eframe::egui::vec2(user_interface.available_width(), content_height),
            Layout::top_down(Align::Min),
            |user_interface| {
                let progress_status_text = install_phase_string(installer_state.installer_phase);
                let header_status_text = installer_status_string(installer_state);

                Frame::new()
                    .fill(self.installer_theme.color_background_panel)
                    .stroke(Stroke::new(1.0, self.installer_theme.color_border_panel))
                    .inner_margin(Margin::same(8))
                    .show(user_interface, |user_interface| {
                        user_interface.style_mut().spacing.item_spacing = eframe::egui::vec2(8.0, 6.0);

                        user_interface.label(
                            RichText::new(header_status_text)
                                .font(self.installer_theme.fonts.font_window_title.clone())
                                .color(self.installer_theme.color_foreground),
                        );

                        Frame::new()
                            .fill(self.installer_theme.color_background_primary)
                            .stroke(Stroke::new(1.0, self.installer_theme.color_border_panel))
                            .inner_margin(Margin::same(8))
                            .show(user_interface, |user_interface| {
                                user_interface.label(
                                    RichText::new("Installation Status")
                                        .font(self.installer_theme.fonts.font_window_title.clone())
                                        .color(self.installer_theme.color_foreground),
                                );
                                user_interface.label(
                                    RichText::new(progress_status_text)
                                        .font(self.installer_theme.fonts.font_normal.clone())
                                        .color(self.installer_theme.color_foreground_preview),
                                );
                                user_interface.add_space(4.0);

                                user_interface.add(
                                    eframe::egui::ProgressBar::new(installer_state.installer_progress)
                                        .fill(if installer_state.install_complete {
                                            self.installer_theme.color_background_control_success
                                        } else {
                                            self.installer_theme.color_background_control_primary
                                        })
                                        .text(installer_state.installer_progress_string.as_str()),
                                );
                            });

                        Frame::new()
                            .fill(self.installer_theme.color_background_primary)
                            .stroke(Stroke::new(1.0, self.installer_theme.color_border_panel))
                            .inner_margin(Margin::same(8))
                            .show(user_interface, |user_interface| {
                                let installer_log_view = InstallerLogView::new(self.installer_theme.clone());
                                installer_log_view.show(user_interface, installer_state);
                            });
                    });
            },
        );

        user_interface.add(installer_footer_view);
        user_interface.style_mut().spacing.item_spacing = previous_item_spacing;
    }
}
