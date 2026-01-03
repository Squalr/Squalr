use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        widgets::controls::{button::Button, state_layer::StateLayer},
    },
    views::project_explorer::project_selector::view_data::project_selector_frame_action::ProjectSelectorFrameAction,
};
use eframe::egui::{Align, Align2, Layout, Rect, Response, Sense, TextureHandle, Ui, Widget, pos2, vec2};
use epaint::{Color32, CornerRadius};
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use std::sync::Arc;

pub struct ProjectEntryView<'lifetime> {
    app_context: Arc<AppContext>,
    project_info: &'lifetime ProjectInfo,
    icon: Option<TextureHandle>,
    project_selector_frame_action: &'lifetime mut ProjectSelectorFrameAction,
}

impl<'lifetime> ProjectEntryView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        project_info: &'lifetime ProjectInfo,
        icon: Option<TextureHandle>,
        project_selector_frame_action: &'lifetime mut ProjectSelectorFrameAction,
    ) -> Self {
        Self {
            app_context: app_context,
            project_info,
            icon,
            project_selector_frame_action,
        }
    }
}

impl<'a> Widget for ProjectEntryView<'a> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let icon_size = vec2(16.0, 16.0);
        let text_left_padding = 4.0;
        let row_height = 28.0;
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_size().x, row_height), Sense::click());

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

        user_interface.with_layout(Layout::left_to_right(Align::Center), |user_interface| {
            let button_size = vec2(36.0, 28.0);

            // Open project.
            let button_refresh = user_interface.add_sized(button_size, Button::new_from_theme(&theme).background_color(Color32::TRANSPARENT));
            IconDraw::draw(user_interface, button_refresh.rect, &theme.icon_library.icon_handle_navigation_refresh);

            if button_refresh.clicked() {
                *self.project_selector_frame_action = ProjectSelectorFrameAction::OpenProject(
                    self.project_info.get_project_directory().unwrap_or_default(),
                    self.project_info.get_name().to_string(),
                );
            }
        });

        response
    }
}
