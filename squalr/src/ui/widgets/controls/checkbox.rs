use crate::ui::widgets::controls::state_layer::StateLayer;
use crate::ui::{theme::Theme, widgets::controls::check_state::CheckState};
use eframe::egui::{Color32, Response, Sense, Ui, Widget};
use epaint::{CornerRadius, Rect, StrokeKind, TextureHandle, Vec2, pos2};

pub struct Checkbox<'lifetime> {
    pub tooltip_text: &'lifetime str,

    pub corner_radius: CornerRadius,
    pub border_width: f32,
    pub size: Vec2,

    pub background_color: Color32,
    pub border_color: Color32,
    pub hover_tint: Color32,
    pub pressed_tint: Color32,
    pub border_color_focused: Option<Color32>,
    pub icon_checked: TextureHandle,
    pub icon_mixed: TextureHandle,

    pub click_sound: Option<&'lifetime str>,

    pub check_state: CheckState,
    pub disabled: bool,
}

impl<'lifetime> Checkbox<'lifetime> {
    pub const WIDTH: f32 = 18.0;
    pub const HEIGHT: f32 = 18.0;

    pub fn new_from_theme(theme: &Theme) -> Self {
        let checkbox = Self {
            tooltip_text: "",

            corner_radius: CornerRadius::ZERO,
            border_width: 1.0,
            size: Vec2::new(18.0, 18.0),

            background_color: theme.background_control,
            border_color: theme.submenu_border,
            hover_tint: theme.hover_tint,
            pressed_tint: theme.pressed_tint,
            border_color_focused: Some(theme.focused_border),
            icon_checked: theme.icon_library.icon_handle_common_check_mark.clone(),
            icon_mixed: theme.icon_library.icon_handle_minimize.clone(),

            click_sound: None,

            check_state: CheckState::False,
            disabled: false,
        };

        checkbox
    }

    pub fn with_check_state(
        mut self,
        check_state: CheckState,
    ) -> Self {
        self.check_state = check_state;
        self
    }

    pub fn with_check_state_bool(
        mut self,
        is_checked: bool,
    ) -> Self {
        self.check_state = CheckState::from_bool(is_checked);
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
        let (allocated_size_rectangle, mut response) = user_interface.allocate_exact_size(self.size, sense);

        // Background box.
        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, self.corner_radius, self.background_color);

        // Border.
        user_interface.painter().rect_stroke(
            allocated_size_rectangle,
            self.corner_radius,
            (self.border_width, self.border_color),
            StrokeKind::Inside,
        );

        // StateLayer (hover, pressed, focus).
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

        // Display icon if checked.
        match self.check_state {
            CheckState::False => {}
            CheckState::True => {
                let texture_size = self.icon_checked.size_vec2();
                let icon_position = allocated_size_rectangle.center() - texture_size * 0.5;

                user_interface.painter().image(
                    self.icon_checked.id(),
                    Rect::from_min_size(icon_position, texture_size),
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
            CheckState::Mixed => {
                let texture_size = self.icon_mixed.size_vec2();
                let icon_position = allocated_size_rectangle.center() - texture_size * 0.5;

                user_interface.painter().image(
                    self.icon_mixed.id(),
                    Rect::from_min_size(icon_position, texture_size),
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
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
