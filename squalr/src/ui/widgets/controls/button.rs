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
    pub hover_color: Color32,
    pub pressed_color: Color32,
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
        button.hover_color = theme.hover_tint;
        button.pressed_color = theme.pressed_tint;
        button.border_color = theme.background_control_primary_dark;
        button.border_color_focused = None;

        button
    }
}

impl<'lifetime> Widget for Button<'lifetime> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let sense = if self.disabled { Sense::hover() } else { Sense::click() };
        let (rect, mut response) = user_interface.allocate_exact_size(user_interface.available_size(), sense);

        // StateLayer compose & paint. This is an overlay to show the hover/focus effect.
        StateLayer {
            bounds_min: rect.min,
            bounds_max: rect.max,
            enabled: !self.disabled,
            pressed: response.is_pointer_button_down_on(),
            has_hover: response.hovered(),
            has_focus: response.has_focus(),

            corner_radius: self.corner_radius,
            border_width: self.border_width,

            hover_color: self.hover_color,
            pressed_color: self.pressed_color,
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
