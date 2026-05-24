use crate::{app_context::AppContext, ui::widgets::controls::groupbox::GroupBox};
use eframe::egui::{Align, Align2, Direction, Key, Layout, RichText, Sense, Stroke, Ui, UiBuilder, pos2, vec2};
use epaint::CornerRadius;
use std::{path::Path, sync::Arc};

pub struct ProjectDeleteConfirmationTakeOverView<'lifetime> {
    app_context: Arc<AppContext>,
    project_directory_path: &'lifetime Path,
    project_name: &'lifetime str,
}

pub struct ProjectDeleteConfirmationTakeOverResponse {
    pub should_cancel: bool,
    pub should_delete: bool,
}

impl<'lifetime> ProjectDeleteConfirmationTakeOverView<'lifetime> {
    const HEADER_HEIGHT: f32 = 32.0;
    const HEADER_TITLE_PADDING_X: f32 = 8.0;
    const CONTENT_PADDING_X: f32 = 12.0;
    const CONTENT_PADDING_TOP: f32 = 12.0;
    const ROW_SPACING: f32 = 8.0;
    const BUTTON_WIDTH: f32 = 120.0;
    const BUTTON_HEIGHT: f32 = 28.0;
    const BUTTON_SPACING: f32 = 12.0;

    pub fn new(
        app_context: Arc<AppContext>,
        project_directory_path: &'lifetime Path,
        project_name: &'lifetime str,
    ) -> Self {
        Self {
            app_context,
            project_directory_path,
            project_name,
        }
    }

    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> ProjectDeleteConfirmationTakeOverResponse {
        let theme = &self.app_context.theme;
        let mut should_cancel = false;
        let mut should_delete = false;
        let (panel_rect, _) = user_interface.allocate_exact_size(user_interface.available_size(), Sense::hover());

        user_interface
            .painter()
            .rect_filled(panel_rect, CornerRadius::ZERO, theme.background_panel);

        let mut panel_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(panel_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        panel_user_interface.set_clip_rect(panel_rect);

        let (header_rect, _) =
            panel_user_interface.allocate_exact_size(vec2(panel_user_interface.available_width().max(1.0), Self::HEADER_HEIGHT), Sense::hover());
        panel_user_interface
            .painter()
            .rect_filled(header_rect, CornerRadius::ZERO, theme.background_primary);
        panel_user_interface.painter().text(
            pos2(header_rect.left() + Self::HEADER_TITLE_PADDING_X, header_rect.center().y),
            Align2::LEFT_CENTER,
            "Delete project",
            theme.font_library.font_noto_sans.font_window_title.clone(),
            theme.foreground,
        );

        panel_user_interface.add_space(Self::CONTENT_PADDING_TOP);
        panel_user_interface.horizontal(|user_interface| {
            user_interface.add_space(Self::CONTENT_PADDING_X);
            user_interface.allocate_ui_with_layout(
                vec2(
                    (user_interface.available_width() - Self::CONTENT_PADDING_X * 2.0).max(0.0),
                    user_interface.available_height(),
                ),
                Layout::top_down(Align::Min),
                |user_interface| {
                    user_interface.allocate_ui_with_layout(
                        user_interface.available_size(),
                        Layout::centered_and_justified(Direction::TopDown),
                        |user_interface| {
                            user_interface.add(
                                GroupBox::new_from_theme(theme, "Confirmation", |user_interface| {
                                    user_interface.vertical_centered(|user_interface| {
                                        user_interface.label(
                                            RichText::new("Delete this project?")
                                                .font(theme.font_library.font_noto_sans.font_header.clone())
                                                .color(theme.foreground),
                                        );
                                        user_interface.add_space(Self::ROW_SPACING);
                                        user_interface.label(
                                            RichText::new(self.project_name)
                                                .font(theme.font_library.font_ubuntu_mono_bold.font_header.clone())
                                                .color(theme.foreground),
                                        );
                                        user_interface.add_space(Self::ROW_SPACING);
                                        user_interface.label(RichText::new(self.project_directory_path.display().to_string()).color(theme.foreground_preview));
                                        user_interface.add_space(Self::ROW_SPACING);
                                        user_interface
                                            .label(RichText::new("This permanently removes the project directory and its files.").color(theme.warning));
                                    });

                                    user_interface.add_space(12.0);
                                    user_interface.allocate_ui(vec2(user_interface.available_width(), 32.0), |user_interface| {
                                        let total_button_row_width = Self::BUTTON_WIDTH * 2.0 + Self::BUTTON_SPACING;
                                        let side_spacing = ((user_interface.available_width() - total_button_row_width) * 0.5).max(0.0);

                                        user_interface.horizontal(|user_interface| {
                                            user_interface.add_space(side_spacing);
                                            user_interface.spacing_mut().item_spacing.x = Self::BUTTON_SPACING;

                                            let cancel_response = user_interface.add_sized(
                                                vec2(Self::BUTTON_WIDTH, Self::BUTTON_HEIGHT),
                                                eframe::egui::Button::new(RichText::new("Cancel").color(theme.foreground))
                                                    .fill(theme.background_control_primary)
                                                    .stroke(Stroke::new(1.0, theme.background_control_primary_dark)),
                                            );
                                            if cancel_response.clicked() {
                                                should_cancel = true;
                                            }

                                            let delete_response = user_interface.add_sized(
                                                vec2(Self::BUTTON_WIDTH, Self::BUTTON_HEIGHT),
                                                eframe::egui::Button::new(RichText::new("Delete").color(theme.foreground))
                                                    .fill(theme.background_control_danger)
                                                    .stroke(Stroke::new(1.0, theme.background_control_danger_dark)),
                                            );
                                            if delete_response.clicked() {
                                                should_delete = true;
                                            }
                                        });
                                    });
                                })
                                .desired_width(user_interface.available_width()),
                            );
                        },
                    );
                },
            );
        });

        if user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) {
            should_cancel = true;
        }

        ProjectDeleteConfirmationTakeOverResponse { should_cancel, should_delete }
    }
}
