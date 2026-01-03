use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        widgets::controls::{button::Button, state_layer::StateLayer},
    },
    views::project_explorer::project_selector::view_data::project_selector_frame_action::ProjectSelectorFrameAction,
};
use eframe::egui::{Align, Align2, Layout, Rect, Response, Sense, TextureHandle, Ui, UiBuilder, Widget, pos2, vec2};
use epaint::{Color32, CornerRadius, Stroke, StrokeKind};
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use std::sync::Arc;

pub struct ProjectEntryView<'lifetime> {
    app_context: Arc<AppContext>,
    project_info: &'lifetime ProjectInfo,
    icon: Option<TextureHandle>,
    is_selected: bool,
    project_selector_frame_action: &'lifetime mut ProjectSelectorFrameAction,
}

impl<'lifetime> ProjectEntryView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        project_info: &'lifetime ProjectInfo,
        icon: Option<TextureHandle>,
        is_selected: bool,
        project_selector_frame_action: &'lifetime mut ProjectSelectorFrameAction,
    ) -> Self {
        Self {
            app_context: app_context,
            project_info,
            icon,
            is_selected,
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
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_size().x, row_height), Sense::click());

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

        if response.clicked() {
            *self.project_selector_frame_action = ProjectSelectorFrameAction::SelectProject(self.project_info.get_project_file_path().to_path_buf());
        }

        // Draw icon and label inside layout.
        let icon_pos_x = allocated_size_rectangle.min.x;
        let icon_pos_y = allocated_size_rectangle.center().y - icon_size.y * 0.5;
        let icon_rect = Rect::from_min_size(pos2(icon_pos_x, icon_pos_y), icon_size);
        let text_pos = pos2(icon_rect.max.x + text_left_padding, allocated_size_rectangle.center().y);

        if let Some(icon) = &self.icon {
            IconDraw::draw_sized(user_interface, icon_rect.center(), icon_size, icon);
        }

        user_interface.painter().text(
            text_pos,
            Align2::LEFT_CENTER,
            self.project_info.get_name(),
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );

        let button_size = vec2(36.0, 28.0);
        let button_rectangle = Rect::from_min_size(
            pos2(allocated_size_rectangle.max.x - button_size.x, allocated_size_rectangle.min.y),
            button_size,
        );

        user_interface.scope_builder(
            UiBuilder::new()
                .max_rect(button_rectangle)
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

        response
    }
}
