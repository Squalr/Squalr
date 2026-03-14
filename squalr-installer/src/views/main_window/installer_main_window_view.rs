use crate::installer_runtime::{install_phase_string, installer_status_string};
use crate::theme::InstallerTheme;
use crate::ui_assets::InstallerIconLibrary;
use crate::ui_state::InstallerUiState;
use crate::views::main_window::installer_footer_view::InstallerFooterView;
use crate::views::main_window::installer_log_view::InstallerLogView;
use crate::views::main_window::installer_title_bar_view::InstallerTitleBarView;
use crate::widgets::installer_checkbox::InstallerCheckbox;
use eframe::egui::{Align, Frame, Layout, Margin, RichText, Stroke, TextEdit, Ui};

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
        installer_state: &mut InstallerUiState,
    ) {
        let previous_item_spacing = user_interface.style().spacing.item_spacing;
        user_interface.style_mut().spacing.item_spacing = eframe::egui::vec2(0.0, 0.0);

        let installer_footer_view = InstallerFooterView::new(
            self.installer_theme.clone(),
            installer_state.install_complete,
            installer_state.install_directory_input.clone(),
        );
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

                        if installer_state.install_started {
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
                        } else {
                            Frame::new()
                                .fill(self.installer_theme.color_background_primary)
                                .stroke(Stroke::new(1.0, self.installer_theme.color_border_panel))
                                .inner_margin(Margin::same(8))
                                .show(user_interface, |user_interface| {
                                    user_interface.label(
                                        RichText::new("Ready to Install")
                                            .font(self.installer_theme.fonts.font_window_title.clone())
                                            .color(self.installer_theme.color_foreground),
                                    );
                                    user_interface.label(
                                        RichText::new("Choose an installation directory and installer options, then confirm installation.")
                                            .font(self.installer_theme.fonts.font_normal.clone())
                                            .color(self.installer_theme.color_foreground_preview),
                                    );
                                    user_interface.add_space(4.0);
                                    user_interface.label(
                                        RichText::new("Installation directory")
                                            .font(self.installer_theme.fonts.font_normal.clone())
                                            .color(self.installer_theme.color_foreground),
                                    );
                                    let install_directory_editor = TextEdit::singleline(&mut installer_state.install_directory_input)
                                        .hint_text("C:\\Users\\<user>\\AppData\\Local\\Programs\\Squalr");
                                    user_interface.add(install_directory_editor);

                                    user_interface.add_space(8.0);
                                    user_interface.label(
                                        RichText::new("Installer options")
                                            .font(self.installer_theme.fonts.font_normal.clone())
                                            .color(self.installer_theme.color_foreground),
                                    );
                                    user_interface.add(InstallerCheckbox::new(
                                        self.installer_theme.clone(),
                                        "Register Squalr in Start Menu",
                                        &mut installer_state
                                            .install_shortcut_options
                                            .register_start_menu_shortcut,
                                    ));
                                    user_interface.add(InstallerCheckbox::new(
                                        self.installer_theme.clone(),
                                        "Create desktop shortcut",
                                        &mut installer_state.install_shortcut_options.create_desktop_shortcut,
                                    ));

                                    user_interface.add_space(8.0);
                                    user_interface.label(
                                        RichText::new("Warning: Existing contents in the selected installation directory will be deleted before install.")
                                            .font(self.installer_theme.fonts.font_normal.clone())
                                            .color(self.installer_theme.color_foreground_warning),
                                    );

                                    if let Some(configuration_error) = installer_state.install_configuration_error.as_ref() {
                                        user_interface.label(
                                            RichText::new(configuration_error)
                                                .font(self.installer_theme.fonts.font_normal.clone())
                                                .color(self.installer_theme.color_foreground_error),
                                        );
                                    }

                                    user_interface.add_space(4.0);
                                    if user_interface.button("Install Squalr").clicked() {
                                        installer_state.install_permission_granted = true;
                                    }
                                });
                        }

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
