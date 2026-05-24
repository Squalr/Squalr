use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        widgets::controls::{button::Button, state_layer::StateLayer},
    },
    views::project_explorer::project_selector::view_data::project_selector_frame_action::ProjectSelectorFrameAction,
};
use eframe::egui::{
    Align, Key, Label, Layout, Response, RichText, Sense, TextEdit, TextureHandle, Ui, UiBuilder, Widget,
    text::{CCursor, CCursorRange},
    vec2,
};
use epaint::{Color32, CornerRadius, Stroke, StrokeKind};
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use std::sync::{Arc, RwLock};

pub struct ProjectEntryView<'lifetime> {
    app_context: Arc<AppContext>,
    project_info: &'lifetime ProjectInfo,
    icon: Option<TextureHandle>,
    is_selected: bool,
    is_renaming: bool,
    rename_project_text: &'lifetime Arc<RwLock<(String, bool)>>,
    project_selector_frame_action: &'lifetime mut ProjectSelectorFrameAction,
}

impl<'lifetime> ProjectEntryView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        project_info: &'lifetime ProjectInfo,
        icon: Option<TextureHandle>,
        is_selected: bool,
        is_renaming: bool,
        rename_project_text: &'lifetime Arc<RwLock<(String, bool)>>,
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
        let row_height = 32.0;
        let button_size = vec2(36.0, row_height);
        let desired_size = vec2(user_interface.available_width(), row_height);
        let (available_size_id, available_size_rect) = user_interface.allocate_space(desired_size);
        let response = user_interface.interact(available_size_rect, available_size_id, Sense::click());

        // Draw selected background and border if applicable.
        if self.is_selected {
            user_interface
                .painter()
                .rect_filled(available_size_rect, CornerRadius::ZERO, theme.selected_background);
            user_interface.painter().rect_stroke(
                available_size_rect,
                CornerRadius::ZERO,
                Stroke::new(1.0, theme.selected_border),
                StrokeKind::Inside,
            );
        }

        // State overlay.
        StateLayer {
            bounds_min: available_size_rect.min,
            bounds_max: available_size_rect.max,
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

        // Handle clicks and double-clicks on the overall area.
        if !self.is_renaming && response.double_clicked() {
            *self.project_selector_frame_action = ProjectSelectorFrameAction::OpenProject(
                self.project_info.get_project_directory().unwrap_or_default(),
                self.project_info.get_name().to_string(),
            );
        } else if !self.is_renaming && response.clicked() {
            *self.project_selector_frame_action = ProjectSelectorFrameAction::SelectProject(self.project_info.get_project_file_path().to_path_buf());
        }

        // Add contents using a bounded UI scope.
        let builder = UiBuilder::new().max_rect(available_size_rect);

        user_interface.scope_builder(builder, |user_interface| {
            user_interface.horizontal(|user_interface| {
                user_interface.set_height(row_height);
                // Rename or cancel button.
                if self.is_renaming {
                    let button_cancel_rename = Button::new_from_theme(theme).background_color(Color32::TRANSPARENT);
                    let response_cancel = user_interface.add_sized(button_size, button_cancel_rename);

                    IconDraw::draw(user_interface, response_cancel.rect, &theme.icon_library.icon_handle_navigation_cancel);

                    if response_cancel.clicked() {
                        *self.project_selector_frame_action = ProjectSelectorFrameAction::CancelRenamingProject();
                    }

                    let button_commit_rename = Button::new_from_theme(theme).background_color(Color32::TRANSPARENT);
                    let response_commit = user_interface.add_sized(button_size, button_commit_rename);

                    IconDraw::draw(user_interface, response_commit.rect, &theme.icon_library.icon_handle_common_check_mark);

                    if response_commit.clicked() {
                        let mut rename_project_guard = match self.rename_project_text.write() {
                            Ok(rename_project_text) => rename_project_text,
                            Err(error) => {
                                log::error!("Failed to acquire rename project text: {}", error);
                                return;
                            }
                        };
                        let rename_project_text = &mut rename_project_guard.0;

                        *self.project_selector_frame_action = ProjectSelectorFrameAction::CommitRename(
                            self.project_info.get_project_file_path().to_path_buf(),
                            std::mem::take(rename_project_text),
                        );
                    }
                }

                // Icon + text/label.
                let trailing_edit_button_width = if self.is_renaming { 0.0 } else { button_size.x };
                let project_name_width = (user_interface.available_width() - trailing_edit_button_width).max(0.0);
                user_interface.allocate_ui_with_layout(vec2(project_name_width, row_height), Layout::left_to_right(Align::Center), |user_interface| {
                    // Draw icon if present.
                    if let Some(icon) = &self.icon {
                        let (_allocated_rect, icon_resp) = user_interface.allocate_exact_size(icon_size, Sense::hover());
                        IconDraw::draw_sized(user_interface, icon_resp.rect.center(), icon_size, icon);
                    }
                    user_interface.add_space(text_left_padding);

                    if self.is_renaming {
                        let mut rename_project_guard = match self.rename_project_text.write() {
                            Ok(rename_project_text) => rename_project_text,
                            Err(error) => {
                                log::error!("Failed to acquire rename project text: {}", error);
                                return;
                            }
                        };
                        let (rename_project_text, should_highlight_text) = &mut *rename_project_guard;
                        let text_edit = TextEdit::singleline(rename_project_text)
                            .font(theme.font_library.font_noto_sans.font_normal.clone())
                            .background_color(theme.background_control)
                            .text_color(theme.foreground)
                            .desired_width(f32::INFINITY);
                        let mut output = text_edit.show(user_interface);
                        let text_edit_response = output.response;

                        if *should_highlight_text {
                            let len_chars = rename_project_text.chars().count();

                            text_edit_response.request_focus();
                            output
                                .state
                                .cursor
                                .set_char_range(Some(CCursorRange::two(CCursor::new(0), CCursor::new(len_chars))));
                            output.state.store(user_interface.ctx(), text_edit_response.id);
                            *should_highlight_text = false;
                        }

                        if text_edit_response.lost_focus() && user_interface.input(|input_state| input_state.key_pressed(Key::Enter)) {
                            *self.project_selector_frame_action = ProjectSelectorFrameAction::CommitRename(
                                self.project_info.get_project_file_path().to_path_buf(),
                                std::mem::take(rename_project_text),
                            );
                        }
                    } else {
                        user_interface.add(
                            Label::new(
                                RichText::new(self.project_info.get_name())
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.foreground),
                            )
                            .selectable(false),
                        );
                        user_interface.add_space(user_interface.available_width());
                    }
                });

                if !self.is_renaming {
                    let edit_response = user_interface.add_sized(
                        button_size,
                        Button::new_from_theme(theme)
                            .background_color(Color32::TRANSPARENT)
                            .with_tooltip_text("Edit project."),
                    );
                    IconDraw::draw(user_interface, edit_response.rect, &theme.icon_library.icon_handle_common_edit);

                    if edit_response.clicked() {
                        *self.project_selector_frame_action = ProjectSelectorFrameAction::StartEditingProject(
                            self.project_info.get_project_file_path().to_path_buf(),
                            self.project_info.get_name().to_string(),
                        );
                    }
                }
            });
        });

        response
    }
}
