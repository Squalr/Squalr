use crate::{
    app_context::AppContext,
    ui::widgets::controls::{checkbox::Checkbox, state_layer::StateLayer},
};
use eframe::egui::{Align2, Rect, Response, Sense, Ui, Widget, pos2, vec2};
use epaint::CornerRadius;
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
        let icon_size = vec2(24.0, 24.0);
        let icon_left_padding = 4.0;
        let text_left_padding = 2.0;

        // Whole clickable area includes indentation.
        let row_height = 32.0;
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(self.width, row_height), Sense::click());

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

        let checkbox_pos_x = icon_left_padding + allocated_size_rectangle.min.x;
        let checkbox_pos_y = allocated_size_rectangle.center().y - icon_size.y * 0.5;
        let checkbox_rect = Rect::from_min_size(pos2(checkbox_pos_x, checkbox_pos_y), icon_size);
        let text_pos = pos2(checkbox_rect.max.x + icon_left_padding + text_left_padding, allocated_size_rectangle.center().y);

        // Draw icon and label inside layout.
        if let Some(is_checked) = self.check_state {
            user_interface.add(Checkbox::new_from_theme(theme).checked(is_checked));
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
