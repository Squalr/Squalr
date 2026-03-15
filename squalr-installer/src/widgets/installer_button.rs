use crate::theme::InstallerTheme;
use crate::widgets::state_layer::StateLayer;
use eframe::egui::{Align2, Color32, CornerRadius, FontId, Response, Sense, Ui, Widget};

pub(crate) struct InstallerButton<'label> {
    label: &'label str,
    disabled: bool,
    corner_radius: CornerRadius,
    border_width: f32,
    background_color: Color32,
    hover_tint: Color32,
    pressed_tint: Color32,
    border_color: Color32,
    border_color_focused: Option<Color32>,
    text_color: Color32,
    disabled_text_color: Color32,
    font_id: FontId,
    sense: Sense,
}

impl<'label> InstallerButton<'label> {
    pub(crate) fn new_from_theme(
        installer_theme: &InstallerTheme,
        label: &'label str,
    ) -> Self {
        Self {
            label,
            disabled: false,
            corner_radius: CornerRadius::same(4),
            border_width: 0.0,
            background_color: installer_theme.color_background_control_primary,
            hover_tint: installer_theme.color_hover_tint,
            pressed_tint: installer_theme.color_pressed_tint,
            border_color: installer_theme.color_background_control_primary_dark,
            border_color_focused: None,
            text_color: installer_theme.color_foreground,
            disabled_text_color: installer_theme.color_foreground_preview,
            font_id: installer_theme.fonts.font_normal.clone(),
            sense: Sense::click(),
        }
    }

    pub(crate) fn background_color(
        mut self,
        background_color: Color32,
    ) -> Self {
        self.background_color = background_color;
        self
    }

    pub(crate) fn border_color(
        mut self,
        border_color: Color32,
    ) -> Self {
        self.border_color = border_color;
        self
    }
}

impl Widget for InstallerButton<'_> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let sense = if self.disabled { Sense::hover() } else { self.sense };
        let (button_rectangle, response) = user_interface.allocate_exact_size(user_interface.available_size(), sense);

        user_interface
            .painter()
            .rect_filled(button_rectangle, self.corner_radius, self.background_color);

        StateLayer {
            bounds_min: button_rectangle.min,
            bounds_max: button_rectangle.max,
            enabled: !self.disabled,
            pressed: response.is_pointer_button_down_on(),
            has_hover: response.hovered(),
            has_focus: response.has_focus(),
            corner_radius: self.corner_radius,
            border_width: self.border_width,
            hover_color: self.hover_tint,
            pressed_color: self.pressed_tint,
            border_color: self.border_color,
            border_color_focused: self.border_color_focused.unwrap_or(self.border_color),
        }
        .ui(user_interface);

        user_interface.painter().text(
            button_rectangle.center(),
            Align2::CENTER_CENTER,
            self.label,
            self.font_id,
            if self.disabled { self.disabled_text_color } else { self.text_color },
        );

        response
    }
}
