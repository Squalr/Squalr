use crate::{
    app_context::AppContext,
    ui::widgets::controls::{checkbox::Checkbox, state_layer::StateLayer},
};
use eframe::egui::{Align2, Rect, Response, Sense, Ui, Widget, pos2, vec2};
use epaint::{Color32, CornerRadius, StrokeKind};
use std::rc::Rc;

/// A generic context menu item.
pub struct ToolbarMenuItemView<'lifetime> {
    studio_context: Rc<AppContext>,
    label: &'lifetime str,
    check_state: Option<bool>,
    width: f32,
}

impl<'lifetime> ToolbarMenuItemView<'lifetime> {
    pub fn new(
        studio_context: Rc<AppContext>,
        label: &'lifetime str,
        check_state: Option<bool>,
        width: f32,
    ) -> Self {
        Self {
            studio_context,
            label,
            check_state,
            width,
        }
    }

    pub fn width(
        mut self,
        width: f32,
    ) -> Self {
        self.width = width;
        self
    }
}

impl<'a> Widget for ToolbarMenuItemView<'a> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.studio_context.theme;
        let icon_size = vec2(18.0, 18.0);
        let icon_left_padding = 4.0;
        let text_left_padding = 2.0;
        let row_height = 32.0;
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(self.width, row_height), Sense::click());

        // Background + overlay.
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

        // Checkbox Overlay Drawing (manual, no layout allocation).
        if let Some(is_checked) = self.check_state {
            let checkbox_pos = pos2(
                allocated_size_rectangle.min.x + icon_left_padding,
                allocated_size_rectangle.center().y - icon_size.y * 0.5,
            );
            let checkbox_rect = Rect::from_min_size(checkbox_pos, icon_size);

            // Draw checkbox background
            user_interface
                .painter()
                .rect_filled(checkbox_rect, CornerRadius::ZERO, theme.background_control);
            user_interface
                .painter()
                .rect_stroke(checkbox_rect, CornerRadius::ZERO, (1.0, theme.submenu_border), StrokeKind::Inside);

            // Draw hover/pressed state
            if response.hovered() {
                user_interface
                    .painter()
                    .rect_filled(checkbox_rect, CornerRadius::ZERO, theme.hover_tint);
            }
            if response.is_pointer_button_down_on() {
                user_interface
                    .painter()
                    .rect_filled(checkbox_rect, CornerRadius::ZERO, theme.pressed_tint);
            }

            // Draw checkmark if checked
            if is_checked {
                let icon = &theme.icon_library.icon_handle_check_mark;
                let texture_size = icon.size_vec2();
                let icon_position = checkbox_rect.center() - texture_size * 0.5;
                user_interface.painter().image(
                    icon.id(),
                    Rect::from_min_size(icon_position, texture_size),
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
        }

        let text_pos = pos2(
            allocated_size_rectangle.min.x + icon_size.x + icon_left_padding * 2.0 + text_left_padding,
            allocated_size_rectangle.center().y,
        );

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
