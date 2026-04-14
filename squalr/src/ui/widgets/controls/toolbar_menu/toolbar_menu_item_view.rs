use crate::{app_context::AppContext, ui::widgets::controls::state_layer::StateLayer};
use eframe::egui::{Align2, FontId, Rect, Response, Sense, Ui, Widget, pos2, vec2};
use epaint::{Color32, CornerRadius, StrokeKind};
use std::sync::Arc;

/// A generic context menu item.
pub struct ToolbarMenuItemView<'lifetime> {
    app_context: Arc<AppContext>,
    label: &'lifetime str,
    item_id: &'lifetime str,
    check_state: &'lifetime Option<Box<dyn Fn() -> Option<bool> + Send + Sync>>,
    width: f32,
}

impl<'lifetime> ToolbarMenuItemView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        label: &'lifetime str,
        item_id: &'lifetime str,
        check_state: &'lifetime Option<Box<dyn Fn() -> Option<bool> + Send + Sync>>,
        width: f32,
    ) -> Self {
        Self {
            app_context,
            label,
            item_id,
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

    pub fn get_item_id(&self) -> &str {
        &self.item_id
    }
}

impl<'a> Widget for ToolbarMenuItemView<'a> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let icon_size = vec2(18.0, 18.0);
        let icon_left_padding = 4.0;
        let text_left_padding = 2.0;
        let text_right_padding = 8.0;
        let row_height = 32.0;
        let row_width = Self::resolve_row_width(
            user_interface,
            self.label,
            &theme.font_library.font_noto_sans.font_normal,
            theme.foreground,
            self.width.max(user_interface.available_width()),
            icon_size.x + icon_left_padding * 2.0 + text_left_padding + text_right_padding,
        );
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(row_width, row_height), Sense::click());

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
        if let Some(check_state) = self.check_state {
            if let Some(is_checked) = check_state() {
                let checkbox_pos = pos2(
                    allocated_size_rectangle.min.x + icon_left_padding,
                    allocated_size_rectangle.center().y - icon_size.y * 0.5,
                );
                let checkbox_rect = Rect::from_min_size(checkbox_pos, icon_size);

                // Draw checkbox background.
                user_interface
                    .painter()
                    .rect_filled(checkbox_rect, CornerRadius::ZERO, theme.background_control);
                user_interface
                    .painter()
                    .rect_stroke(checkbox_rect, CornerRadius::ZERO, (1.0, theme.submenu_border), StrokeKind::Inside);

                // Draw hover/pressed state.
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

                // Draw checkmark if checked.
                if is_checked {
                    let icon = &theme.icon_library.icon_handle_common_check_mark;
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
        }

        let text_left = allocated_size_rectangle.min.x + icon_size.x + icon_left_padding * 2.0 + text_left_padding;
        let text_rectangle = Rect::from_min_max(
            pos2(text_left, allocated_size_rectangle.min.y),
            pos2(allocated_size_rectangle.max.x - text_right_padding, allocated_size_rectangle.max.y),
        );
        let text_to_render = Self::truncate_text_to_width(
            user_interface,
            self.label,
            &theme.font_library.font_noto_sans.font_normal,
            theme.foreground,
            text_rectangle.width().max(0.0),
        );
        let text_pos = pos2(text_rectangle.min.x, allocated_size_rectangle.center().y);

        user_interface.painter().with_clip_rect(text_rectangle).text(
            text_pos,
            Align2::LEFT_CENTER,
            text_to_render,
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );

        response
    }
}

impl<'lifetime> ToolbarMenuItemView<'lifetime> {
    fn resolve_row_width(
        user_interface: &mut Ui,
        label: &str,
        font_id: &FontId,
        text_color: Color32,
        minimum_row_width: f32,
        content_padding_width: f32,
    ) -> f32 {
        let content_width = Self::measure_text_width(user_interface, label, font_id, text_color) + content_padding_width;

        Self::resolve_row_width_from_content_width(minimum_row_width, content_width)
    }

    fn resolve_row_width_from_content_width(
        minimum_row_width: f32,
        content_width: f32,
    ) -> f32 {
        minimum_row_width.max(content_width.ceil())
    }

    fn measure_text_width(
        user_interface: &mut Ui,
        text: &str,
        font_id: &FontId,
        text_color: Color32,
    ) -> f32 {
        user_interface.ctx().fonts_mut(|fonts| {
            fonts
                .layout_no_wrap(text.to_string(), font_id.clone(), text_color)
                .size()
                .x
        })
    }

    fn truncate_text_to_width(
        user_interface: &mut Ui,
        label: &str,
        font_id: &FontId,
        text_color: Color32,
        max_text_width: f32,
    ) -> String {
        if max_text_width <= 0.0 {
            return String::new();
        }

        let full_text_width = Self::measure_text_width(user_interface, label, font_id, text_color);
        if full_text_width <= max_text_width {
            return label.to_string();
        }

        let ellipsis = "...";
        let ellipsis_width = Self::measure_text_width(user_interface, ellipsis, font_id, text_color);
        if ellipsis_width > max_text_width {
            return String::new();
        }

        let mut truncated_label = label.to_string();
        while !truncated_label.is_empty() {
            truncated_label.pop();
            let candidate_label = format!("{}{}", truncated_label, ellipsis);
            if Self::measure_text_width(user_interface, &candidate_label, font_id, text_color) <= max_text_width {
                return candidate_label;
            }
        }

        ellipsis.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::ToolbarMenuItemView;

    #[test]
    fn resolve_row_width_from_content_width_keeps_minimum_width_for_short_content() {
        let resolved_row_width = ToolbarMenuItemView::resolve_row_width_from_content_width(160.0, 124.2);

        assert_eq!(resolved_row_width, 160.0);
    }

    #[test]
    fn resolve_row_width_from_content_width_expands_for_long_content() {
        let resolved_row_width = ToolbarMenuItemView::resolve_row_width_from_content_width(160.0, 190.2);

        assert_eq!(resolved_row_width, 191.0);
    }
}
