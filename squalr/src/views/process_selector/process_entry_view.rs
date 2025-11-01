use crate::{
    app_context::AppContext,
    ui::{draw::icon_draw::IconDraw, widgets::controls::state_layer::StateLayer},
};
use eframe::egui::{Align2, Rect, Response, Sense, TextureHandle, Ui, Widget, pos2, vec2};
use epaint::CornerRadius;
use std::sync::Arc;

/// A generic context menu item.
pub struct ProcessEntryView<'lifetime> {
    app_context: Arc<AppContext>,
    label: &'lifetime str,
    icon: Option<TextureHandle>,
}

impl<'lifetime> ProcessEntryView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        label: &'lifetime str,
        icon: Option<TextureHandle>,
    ) -> Self {
        Self {
            app_context: app_context,
            label,
            icon,
        }
    }
}

impl<'a> Widget for ProcessEntryView<'a> {
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
            self.label,
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );

        response
    }
}
