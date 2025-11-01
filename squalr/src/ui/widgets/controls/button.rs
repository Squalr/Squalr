use crate::ui::theme::Theme;
use crate::ui::widgets::controls::state_layer::StateLayer;
use eframe::egui::{Color32, Response, Sense, Ui, Widget};
use epaint::CornerRadius;

#[derive(Default)]
pub struct Button<'lifetime> {
    pub disabled: bool,
    pub tooltip_text: &'lifetime str,
    pub corner_radius: CornerRadius,
    pub border_width: f32,
    pub margin: i8,
    pub backgorund_color: Color32,
    pub hover_tint: Color32,
    pub pressed_tint: Color32,
    pub border_color: Color32,
    pub click_sound: Option<&'lifetime str>,
    pub border_color_focused: Option<Color32>,
}

impl<'lifetime> Button<'lifetime> {
    pub fn new_from_theme(theme: &Theme) -> Button<'lifetime> {
        let mut button = Button::default();

        button.corner_radius = CornerRadius { nw: 4, ne: 4, sw: 4, se: 4 };
        button.border_width = 0.0;
        button.margin = 0;
        button.backgorund_color = theme.background_control_primary;
        button.hover_tint = theme.hover_tint;
        button.pressed_tint = theme.pressed_tint;
        button.border_color = theme.background_control_primary_dark;
        button.border_color_focused = None;

        button
    }

    pub fn disabled(
        mut self,
        disabled: bool,
    ) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn tooltip_text(
        mut self,
        tooltip: &'lifetime str,
    ) -> Self {
        self.tooltip_text = tooltip;
        self
    }

    pub fn corner_radius(
        mut self,
        corner_radius: CornerRadius,
    ) -> Self {
        self.corner_radius = corner_radius;
        self
    }

    pub fn border_width(
        mut self,
        border_width: f32,
    ) -> Self {
        self.border_width = border_width;
        self
    }

    pub fn margin(
        mut self,
        margin: i8,
    ) -> Self {
        self.margin = margin;
        self
    }

    pub fn background_color(
        mut self,
        color: Color32,
    ) -> Self {
        self.backgorund_color = color;
        self
    }

    pub fn hover_color(
        mut self,
        color: Color32,
    ) -> Self {
        self.hover_tint = color;
        self
    }

    pub fn pressed_color(
        mut self,
        color: Color32,
    ) -> Self {
        self.pressed_tint = color;
        self
    }

    pub fn border_color(
        mut self,
        color: Color32,
    ) -> Self {
        self.border_color = color;
        self
    }

    pub fn border_color_focused(
        mut self,
        color: Option<Color32>,
    ) -> Self {
        self.border_color_focused = color;
        self
    }

    pub fn click_sound(
        mut self,
        sound: Option<&'lifetime str>,
    ) -> Self {
        self.click_sound = sound;
        self
    }
}

impl<'lifetime> Widget for Button<'lifetime> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let sense = if self.disabled { Sense::hover() } else { Sense::click() };
        let (allocated_size_rectangle, mut response) = user_interface.allocate_exact_size(user_interface.available_size(), sense);

        // Background.
        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, self.corner_radius, self.backgorund_color);

        // StateLayer compose & paint. This is an overlay to show the hover/focus effect.
        StateLayer {
            bounds_min: allocated_size_rectangle.min,
            bounds_max: allocated_size_rectangle.max,
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

        // Tooltip.
        if !self.tooltip_text.is_empty() {
            response = response.on_hover_text(self.tooltip_text);
        }

        // Click sound.
        if response.clicked() {
            if let Some(sound) = self.click_sound {
                println!("JIRA: Play sound: {}", sound);
            }
        }

        response
    }
}
