use crate::{
    app_context::AppContext,
    ui::{text::text_fitting::truncate_text_to_width, widgets::controls::state_layer::StateLayer},
};
use eframe::egui::{Align2, Rect, Response, Sense, TextureHandle, Ui, Widget, pos2, vec2};
use epaint::{Color32, CornerRadius, StrokeKind};
use std::sync::Arc;

/// A generic context menu item.
pub struct ToolbarMenuItemView<'lifetime> {
    app_context: Arc<AppContext>,
    label: &'lifetime str,
    item_id: &'lifetime str,
    check_state: &'lifetime Option<Box<dyn Fn() -> Option<bool> + Send + Sync>>,
    icon: Option<TextureHandle>,
    icon_background_color: Option<Color32>,
    icon_border_color: Option<Color32>,
    width: f32,
    disabled: bool,
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
            icon: None,
            icon_background_color: None,
            icon_border_color: None,
            width,
            disabled: false,
        }
    }

    pub fn icon(
        mut self,
        icon: TextureHandle,
    ) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn icon_background(
        mut self,
        background_color: Color32,
        border_color: Color32,
    ) -> Self {
        self.icon_background_color = Some(background_color);
        self.icon_border_color = Some(border_color);
        self
    }

    pub fn width(
        mut self,
        width: f32,
    ) -> Self {
        self.width = width;
        self
    }

    pub fn disabled(
        mut self,
        disabled: bool,
    ) -> Self {
        self.disabled = disabled;
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
        let icon_size = vec2(Self::ICON_WIDTH, 18.0);
        let icon_left_padding = Self::ICON_LEFT_PADDING;
        let text_left_padding = Self::TEXT_LEFT_PADDING;
        let text_right_padding = Self::TEXT_RIGHT_PADDING;
        let row_height = 32.0;
        let row_width = self.width;
        let sense = if self.disabled { Sense::hover() } else { Sense::click() };
        let text_color = if self.disabled { theme.foreground_preview } else { theme.foreground };
        let icon_tint = if self.disabled { theme.foreground_preview } else { Color32::WHITE };
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(row_width, row_height), sense);

        // Background + overlay.
        StateLayer {
            bounds_min: allocated_size_rectangle.min,
            bounds_max: allocated_size_rectangle.max,
            enabled: !self.disabled,
            pressed: !self.disabled && response.is_pointer_button_down_on(),
            has_hover: !self.disabled && response.hovered(),
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
                user_interface.painter().rect_stroke(
                    checkbox_rect,
                    CornerRadius::ZERO,
                    (1.0, if self.disabled { theme.foreground_preview } else { theme.submenu_border }),
                    StrokeKind::Inside,
                );

                // Draw hover/pressed state.
                if !self.disabled && response.hovered() {
                    user_interface
                        .painter()
                        .rect_filled(checkbox_rect, CornerRadius::ZERO, theme.hover_tint);
                }
                if !self.disabled && response.is_pointer_button_down_on() {
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
                        icon_tint,
                    );
                }
            }
        } else if let Some(icon) = &self.icon {
            let icon_position = pos2(
                allocated_size_rectangle.min.x + icon_left_padding,
                allocated_size_rectangle.center().y - icon_size.y * 0.5,
            );
            let icon_rect = Rect::from_min_size(icon_position, icon_size);

            if let Some(icon_background_color) = self.icon_background_color {
                user_interface
                    .painter()
                    .rect_filled(icon_rect, CornerRadius::ZERO, icon_background_color);
                user_interface.painter().rect_stroke(
                    icon_rect,
                    CornerRadius::ZERO,
                    (1.0, self.icon_border_color.unwrap_or(icon_background_color)),
                    StrokeKind::Inside,
                );
            }

            user_interface
                .painter()
                .image(icon.id(), icon_rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), icon_tint);
        }

        let text_left = allocated_size_rectangle.min.x + icon_size.x + icon_left_padding * 2.0 + text_left_padding;
        let text_rectangle = Rect::from_min_max(
            pos2(text_left, allocated_size_rectangle.min.y),
            pos2(allocated_size_rectangle.max.x - text_right_padding, allocated_size_rectangle.max.y),
        );
        let text_to_render = truncate_text_to_width(
            user_interface,
            self.label,
            &theme.font_library.font_noto_sans.font_normal,
            text_color,
            text_rectangle.width().max(0.0),
        );
        let text_pos = pos2(text_rectangle.min.x, allocated_size_rectangle.center().y);

        user_interface.painter().with_clip_rect(text_rectangle).text(
            text_pos,
            Align2::LEFT_CENTER,
            text_to_render,
            theme.font_library.font_noto_sans.font_normal.clone(),
            text_color,
        );

        response
    }
}

impl<'lifetime> ToolbarMenuItemView<'lifetime> {
    pub const MIN_MENU_WIDTH: f32 = 160.0;
    const ICON_WIDTH: f32 = 18.0;
    const ICON_LEFT_PADDING: f32 = 4.0;
    const TEXT_LEFT_PADDING: f32 = 2.0;
    const TEXT_RIGHT_PADDING: f32 = 8.0;

    pub fn row_width_from_text_width(text_width: f32) -> f32 {
        (text_width + Self::ICON_WIDTH + Self::ICON_LEFT_PADDING * 2.0 + Self::TEXT_LEFT_PADDING + Self::TEXT_RIGHT_PADDING).max(Self::MIN_MENU_WIDTH)
    }
}
