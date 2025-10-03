use crate::ui::theme::Theme;
use crate::ui::widgets::controls::state_layer::StateLayer;
use eframe::egui::{Color32, Response, Sense, Ui, Widget};
use epaint::{CornerRadius, Rect, StrokeKind, TextureHandle, Vec2, pos2};

#[derive(Default)]
pub struct Checkbox<'lifetime> {
    pub is_checked: bool,
    pub disabled: bool,
    pub tooltip_text: &'lifetime str,

    pub corner_radius: CornerRadius,
    pub border_width: f32,
    pub size: Vec2,

    pub background_color: Color32,
    pub border_color: Color32,
    pub hover_tint: Color32,
    pub pressed_tint: Color32,
    pub border_color_focused: Option<Color32>,
    pub icon: Option<TextureHandle>,

    pub click_sound: Option<&'lifetime str>,
}

impl<'lifetime> Checkbox<'lifetime> {
    pub fn new_from_theme(theme: &Theme) -> Self {
        let mut checkbox = Checkbox::default();

        checkbox.corner_radius = CornerRadius::ZERO;
        checkbox.border_width = 1.0;
        checkbox.size = Vec2::new(18.0, 18.0);

        checkbox.background_color = theme.background_control;
        checkbox.border_color = theme.submenu_border;
        checkbox.hover_tint = theme.hover_tint;
        checkbox.pressed_tint = theme.pressed_tint;
        checkbox.border_color_focused = Some(theme.focused_border);
        checkbox.icon = Some(theme.icon_library.icon_handle_check_mark.clone());

        checkbox
    }

    pub fn checked(
        mut self,
        checked: bool,
    ) -> Self {
        self.is_checked = checked;
        self
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
}

impl<'lifetime> Widget for Checkbox<'lifetime> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let sense = if self.disabled { Sense::hover() } else { Sense::click() };
        let (available_size_rectangle, mut response) = user_interface.allocate_exact_size(self.size, sense);

        // Background box.
        user_interface
            .painter()
            .rect_filled(available_size_rectangle, self.corner_radius, self.background_color);

        // Border.
        user_interface.painter().rect_stroke(
            available_size_rectangle,
            self.corner_radius,
            (self.border_width, self.border_color),
            StrokeKind::Inside,
        );

        // StateLayer (hover, pressed, focus).
        StateLayer {
            bounds_min: available_size_rectangle.min,
            bounds_max: available_size_rectangle.max,
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

        // Checked icon (using theme’s close icon for now).
        if self.is_checked {
            if let Some(icon) = self.icon {
                let texture_size = icon.size_vec2();
                let icon_position = available_size_rectangle.center() - texture_size * 0.5;

                user_interface.painter().image(
                    icon.id(),
                    Rect::from_min_size(icon_position, texture_size),
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(texture_size.x, texture_size.y)),
                    Color32::WHITE,
                );
            }
        }

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
