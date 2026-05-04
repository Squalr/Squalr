use crate::{app_context::AppContext, ui::widgets::controls::state_layer::StateLayer};
use eframe::egui::{Align2, Rect, Response, Sense, TextureHandle, Ui, Widget, pos2, vec2};
use epaint::CornerRadius;
use std::sync::Arc;

/// A generic context menu item.
pub struct ComboBoxItemView<'lifetime> {
    app_context: Arc<AppContext>,
    label: &'lifetime str,
    icon: Option<TextureHandle>,
    combo_box_width: f32,
}

impl<'lifetime> ComboBoxItemView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        label: &'lifetime str,
        icon: Option<TextureHandle>,
        width: f32,
    ) -> Self {
        Self {
            app_context: app_context,
            label,
            icon,
            combo_box_width: width,
        }
    }

    pub fn width(
        mut self,
        width: f32,
    ) -> Self {
        self.combo_box_width = width;
        self
    }
}

impl<'a> Widget for ComboBoxItemView<'a> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let icon_size = vec2(16.0, 16.0);
        let icon_left_padding = 8.0;
        let text_left_padding = 0.0;

        // Whole clickable area includes indentation.
        let row_height = 28.0;
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(self.combo_box_width, row_height), Sense::click());

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
        let content_clip_rectangle = allocated_size_rectangle.intersect(user_interface.clip_rect());
        let text_start_position = if let Some(icon) = &self.icon {
            let icon_pos_x = allocated_size_rectangle.min.x + icon_left_padding;
            let icon_pos_y = allocated_size_rectangle.center().y - icon_size.y * 0.5;
            let icon_rect = Rect::from_min_size(pos2(icon_pos_x, icon_pos_y), icon_size);

            user_interface
                .painter()
                .with_clip_rect(content_clip_rectangle)
                .image(icon.id(), icon_rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), epaint::Color32::WHITE);

            pos2(icon_rect.max.x + icon_left_padding + text_left_padding, allocated_size_rectangle.min.y)
        } else {
            pos2(allocated_size_rectangle.min.x + icon_left_padding, allocated_size_rectangle.min.y)
        };
        let text_width = (allocated_size_rectangle.max.x - text_start_position.x - icon_left_padding).max(0.0);
        let text_rectangle = Rect::from_min_size(text_start_position, vec2(text_width, row_height));

        let text_to_render = Self::truncate_text_to_width(
            user_interface,
            self.label,
            text_width,
            &theme.font_library.font_noto_sans.font_normal,
            theme.foreground,
        );
        let text_painter = user_interface
            .painter()
            .with_clip_rect(text_rectangle.intersect(content_clip_rectangle));
        text_painter.text(
            pos2(text_rectangle.min.x, allocated_size_rectangle.center().y),
            Align2::LEFT_CENTER,
            text_to_render,
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );

        response
    }
}

impl<'lifetime> ComboBoxItemView<'lifetime> {
    fn truncate_text_to_width(
        user_interface: &Ui,
        label: &str,
        max_text_width: f32,
        font_id: &eframe::egui::FontId,
        text_color: epaint::Color32,
    ) -> String {
        if max_text_width <= 0.0 {
            return String::new();
        }

        let full_text_width = user_interface.ctx().fonts_mut(|fonts| {
            fonts
                .layout_no_wrap(label.to_string(), font_id.clone(), text_color)
                .size()
                .x
        });
        if full_text_width <= max_text_width {
            return label.to_string();
        }

        let ellipsis = "...";
        let ellipsis_width = user_interface.ctx().fonts_mut(|fonts| {
            fonts
                .layout_no_wrap(ellipsis.to_string(), font_id.clone(), text_color)
                .size()
                .x
        });
        if ellipsis_width > max_text_width {
            return String::new();
        }

        let mut truncated_label = label.to_string();
        while !truncated_label.is_empty() {
            truncated_label.pop();
            let candidate_label = format!("{}{}", truncated_label, ellipsis);
            let candidate_width = user_interface.ctx().fonts_mut(|fonts| {
                fonts
                    .layout_no_wrap(candidate_label.clone(), font_id.clone(), text_color)
                    .size()
                    .x
            });
            if candidate_width <= max_text_width {
                return candidate_label;
            }
        }

        String::new()
    }
}
