use crate::{
    app_context::AppContext,
    ui::{draw::icon_draw::IconDraw, widgets::controls::state_layer::StateLayer},
    views::project_explorer::project_hierarchy::view_data::project_hierarchy_frame_action::ProjectHierarchyFrameAction,
};
use eframe::egui::{Align2, Rect, Response, Sense, TextureHandle, Ui, Widget, pos2, vec2};
use epaint::{CornerRadius, Stroke, StrokeKind};
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use std::sync::Arc;

pub struct ProjectItemEntryView<'lifetime> {
    app_context: Arc<AppContext>,
    project_info: &'lifetime ProjectInfo,
    icon: Option<TextureHandle>,
    is_selected: bool,
    project_hierarchy_frame_action: &'lifetime mut ProjectHierarchyFrameAction,
}

impl<'lifetime> ProjectItemEntryView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        project_info: &'lifetime ProjectInfo,
        icon: Option<TextureHandle>,
        is_selected: bool,
        project_hierarchy_frame_action: &'lifetime mut ProjectHierarchyFrameAction,
    ) -> Self {
        Self {
            app_context: app_context,
            project_info,
            icon,
            is_selected,
            project_hierarchy_frame_action,
        }
    }
}

impl<'lifetime> Widget for ProjectItemEntryView<'lifetime> {
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
            //
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

        response
    }
}
