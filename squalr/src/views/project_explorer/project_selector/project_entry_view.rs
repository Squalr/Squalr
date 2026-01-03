use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        widgets::controls::{button::Button, state_layer::StateLayer},
    },
    views::project_explorer::project_selector::view_data::project_selector_frame_action::ProjectSelectorFrameAction,
};
use eframe::egui::{self, Align, Align2, Layout, Rect, Response, Sense, TextBuffer, TextureHandle, Ui, UiBuilder, Widget, pos2, vec2};
use epaint::{Color32, CornerRadius, Stroke, StrokeKind};
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use std::sync::{Arc, RwLock};

pub struct ProjectEntryView<'lifetime> {
    app_context: Arc<AppContext>,
    project_info: &'lifetime ProjectInfo,
    icon: Option<TextureHandle>,
    is_selected: bool,
    is_renaming: bool,
    rename_project_text: &'lifetime Arc<RwLock<String>>,
    project_selector_frame_action: &'lifetime mut ProjectSelectorFrameAction,
}

impl<'lifetime> ProjectEntryView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        project_info: &'lifetime ProjectInfo,
        icon: Option<TextureHandle>,
        is_selected: bool,
        is_renaming: bool,
        rename_project_text: &'lifetime Arc<RwLock<String>>,
        project_selector_frame_action: &'lifetime mut ProjectSelectorFrameAction,
    ) -> Self {
        Self {
            app_context: app_context,
            project_info,
            icon,
            is_selected,
            is_renaming,
            rename_project_text,
            project_selector_frame_action,
        }
    }
}

impl<'lifetime> Widget for ProjectEntryView<'lifetime> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let icon_size = vec2(16.0, 16.0);
        let text_left_padding = 4.0;
        let row_height = 28.0;
        let button_size = vec2(36.0, 28.0);
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_size().x, row_height), Sense::click());
        let button_open_rect = Rect::from_min_size(allocated_size_rectangle.min, button_size);

        user_interface.scope_builder(
            UiBuilder::new()
                .max_rect(button_open_rect)
                .layout(Layout::left_to_right(Align::Center)),
            |user_interface| {
                // Open project.
                let button_open = user_interface.add_sized(button_size, Button::new_from_theme(&theme).background_color(Color32::TRANSPARENT));

                IconDraw::draw(user_interface, button_open.rect, &theme.icon_library.icon_handle_file_system_open_folder);
                if button_open.clicked() {
                    *self.project_selector_frame_action = ProjectSelectorFrameAction::OpenProject(
                        self.project_info.get_project_directory().unwrap_or_default(),
                        self.project_info.get_name().to_string(),
                    );
                }
            },
        );

        if self.is_selected {
            // Draw the background.
            user_interface
                .painter()
                .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.selected_background);

            // Draw the border.
            user_interface.painter().rect_stroke(
                allocated_size_rectangle,
                CornerRadius::ZERO,
                Stroke::new(1.0, theme.selected_border),
                StrokeKind::Inside,
            );
        }

        // Background and state overlay.
        StateLayer {
            bounds_min: allocated_size_rectangle.min,
            bounds_max: allocated_size_rectangle.max,
            enabled: true,
            pressed: response.is_pointer_button_down_on(),
            has_hover: response.hovered(),
            has_focus: response.has_focus(),
            corner_radius: CornerRadius::ZERO,
            border_width: 0.0,
            hover_color: theme.hover_tint,
            pressed_color: theme.pressed_tint,
            border_color: theme.background_control_secondary_dark,
            border_color_focused: theme.background_control_secondary_dark,
        }
        .ui(user_interface);

        if response.double_clicked() {
            *self.project_selector_frame_action = ProjectSelectorFrameAction::OpenProject(
                self.project_info.get_project_directory().unwrap_or_default(),
                self.project_info.get_name().to_string(),
            );
        } else if response.clicked() {
            *self.project_selector_frame_action = ProjectSelectorFrameAction::SelectProject(self.project_info.get_project_file_path().to_path_buf());
        }

        // Draw icon and label inside layout.
        let icon_pos_x = allocated_size_rectangle.min.x + button_size.x;
        let icon_pos_y = allocated_size_rectangle.center().y - icon_size.y * 0.5;
        let icon_rect = Rect::from_min_size(pos2(icon_pos_x, icon_pos_y), icon_size);
        let text_pos = pos2(icon_rect.max.x + text_left_padding, allocated_size_rectangle.center().y);

        if let Some(icon) = &self.icon {
            IconDraw::draw_sized(user_interface, icon_rect.center(), icon_size, icon);
        }

        if self.is_renaming {
            let mut rename_project_text = match self.rename_project_text.write() {
                Ok(rename_project_text) => rename_project_text,
                Err(error) => {
                    log::error!("Failed to acquire rename project text: {}", error);
                    return response;
                }
            };

            let text_min_x = text_pos.x;
            let text_max_x = allocated_size_rectangle.max.x - button_size.x;
            let text_rect = Rect::from_min_max(
                pos2(text_min_x, allocated_size_rectangle.min.y),
                pos2(text_max_x, allocated_size_rectangle.max.y),
            );
            user_interface.scope_builder(
                UiBuilder::new()
                    .max_rect(text_rect)
                    .layout(Layout::left_to_right(Align::Center)),
                |user_interface| {
                    let text_edit_response = user_interface.add(
                        egui::TextEdit::singleline(&mut *rename_project_text)
                            .font(theme.font_library.font_noto_sans.font_normal.clone())
                            .background_color(theme.background_control)
                            .text_color(theme.foreground),
                    );
                    if text_edit_response.lost_focus() && user_interface.input(|input_state| input_state.key_pressed(egui::Key::Enter)) {
                        *self.project_selector_frame_action =
                            ProjectSelectorFrameAction::CommitRename(self.project_info.get_project_directory().unwrap_or_default(), rename_project_text.take());
                    }
                },
            );
        } else {
            user_interface.painter().text(
                text_pos,
                Align2::LEFT_CENTER,
                self.project_info.get_name(),
                theme.font_library.font_noto_sans.font_normal.clone(),
                theme.foreground,
            );
        }

        let button_rectangle = Rect::from_min_size(
            pos2(allocated_size_rectangle.max.x - button_size.x, allocated_size_rectangle.min.y),
            button_size,
        );

        user_interface.scope_builder(
            UiBuilder::new()
                .max_rect(button_rectangle)
                .layout(Layout::left_to_right(Align::Center)),
            |user_interface| {
                if self.is_renaming {
                    // Cancel rename project.
                    let button_cancel_rename = user_interface.add_sized(button_size, Button::new_from_theme(&theme).background_color(Color32::TRANSPARENT));
                    IconDraw::draw(user_interface, button_cancel_rename.rect, &theme.icon_library.icon_handle_navigation_cancel);

                    if button_cancel_rename.clicked() {
                        *self.project_selector_frame_action = ProjectSelectorFrameAction::CancelRenamingProject();
                    }
                } else {
                    // Rename project.
                    let button_rename = user_interface.add_sized(button_size, Button::new_from_theme(&theme).background_color(Color32::TRANSPARENT));
                    IconDraw::draw(user_interface, button_rename.rect, &theme.icon_library.icon_handle_common_edit);

                    if button_rename.clicked() {
                        *self.project_selector_frame_action = ProjectSelectorFrameAction::StartRenamingProject(
                            self.project_info.get_project_file_path().to_path_buf(),
                            self.project_info.get_name().to_string(),
                        );
                    }
                }
            },
        );
        response
    }
}
