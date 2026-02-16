use crate::{app_context::AppContext, ui::widgets::controls::state_layer::StateLayer};
use eframe::egui::{Align2, Rect, Response, Sense, Ui, Widget, pos2, vec2};
use epaint::{Color32, CornerRadius, StrokeKind};
use squalr_engine_api::structures::data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat};
use std::sync::Arc;

pub struct DataValueBoxConvertItemView<'lifetime> {
    app_context: Arc<AppContext>,
    anonymous_value_string: &'lifetime mut AnonymousValueString,
    target_anonymous_value_string_format: &'lifetime AnonymousValueStringFormat,
    target_display_value: Option<&'lifetime AnonymousValueString>,
    is_conversion_available: bool,
    is_value_owned: bool,
    combo_box_width: f32,
}

impl<'lifetime> DataValueBoxConvertItemView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        anonymous_value_string: &'lifetime mut AnonymousValueString,
        target_anonymous_value_string_format: &'lifetime AnonymousValueStringFormat,
        target_display_value: Option<&'lifetime AnonymousValueString>,
        is_conversion_available: bool,
        is_value_owned: bool,
        width: f32,
    ) -> Self {
        Self {
            app_context: app_context,
            anonymous_value_string,
            target_anonymous_value_string_format,
            target_display_value,
            is_conversion_available,
            is_value_owned,
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

impl<'a> Widget for DataValueBoxConvertItemView<'a> {
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

        // Show a checkbox only for interpretations.
        if !self.is_conversion_available {
            let checkbox_pos = pos2(
                allocated_size_rectangle.min.x + icon_left_padding,
                allocated_size_rectangle.center().y - icon_size.y * 0.5,
            );
            let checkbox_rectangle = Rect::from_min_size(checkbox_pos, icon_size);

            // Draw checkbox background.
            user_interface
                .painter()
                .rect_filled(checkbox_rectangle, CornerRadius::ZERO, theme.background_control);
            user_interface
                .painter()
                .rect_stroke(checkbox_rectangle, CornerRadius::ZERO, (1.0, theme.submenu_border), StrokeKind::Inside);

            // Draw hover/pressed state.
            if response.hovered() {
                user_interface
                    .painter()
                    .rect_filled(checkbox_rectangle, CornerRadius::ZERO, theme.hover_tint);
            }
            if response.is_pointer_button_down_on() {
                user_interface
                    .painter()
                    .rect_filled(checkbox_rectangle, CornerRadius::ZERO, theme.pressed_tint);
            }

            // Draw checkmark if checked.
            if self.anonymous_value_string.get_anonymous_value_string_format() == *self.target_anonymous_value_string_format {
                let icon = &theme.icon_library.icon_handle_common_check_mark;
                let texture_size = icon.size_vec2();
                let icon_position = checkbox_rectangle.center() - texture_size * 0.5;
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

        let text = if self.is_conversion_available {
            format!("Convert to {}", self.target_anonymous_value_string_format)
        } else if self.is_value_owned {
            format!("Interpret as {}", self.target_anonymous_value_string_format)
        } else {
            format!("Display as {}", self.target_anonymous_value_string_format)
        };

        user_interface.painter().text(
            text_pos,
            Align2::LEFT_CENTER,
            text,
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );

        if response.clicked() {
            if self.is_conversion_available {
                self.anonymous_value_string
                    .set_anonymous_value_string_format(*self.target_anonymous_value_string_format);
            } else if let Some(target_display_value) = self.target_display_value {
                *self.anonymous_value_string = target_display_value.clone();
            } else {
                self.anonymous_value_string
                    .set_anonymous_value_string_format(*self.target_anonymous_value_string_format);
            }
        }

        response
    }
}
