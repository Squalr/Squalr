use crate::ui::theme::Theme;
use crate::ui::widgets::controls::state_layer::StateLayer;
use eframe::egui::{Color32, Response, Sense, Ui, Widget};
use epaint::{CornerRadius, Rect, TextureHandle, pos2, vec2};

#[derive(Default)]
pub struct Slider<'lifetime> {
    pub current_value: i64,
    pub minimum_value: i64,
    pub maximum_value: i64,
    pub tooltip_text: &'lifetime str,

    pub background_color_handle: Color32,
    pub background_color_track: Color32,
    pub border_color: Color32,
    pub hover_tint: Color32,
    pub pressed_tint: Color32,
    pub border_color_focused: Option<Color32>,
    pub icon: Option<TextureHandle>,

    pub click_sound: Option<&'lifetime str>,
    pub track_width: f32,
    pub handle_size: u8,
    pub disabled: bool,
}

impl<'lifetime> Slider<'lifetime> {
    pub fn new_from_theme(theme: &Theme) -> Self {
        let mut slider = Slider::default();

        slider.handle_size = 20;
        slider.track_width = 128.0;

        slider.background_color_handle = theme.background_control_primary;
        slider.background_color_track = theme.background_control;
        slider.border_color = theme.submenu_border;
        slider.hover_tint = theme.hover_tint;
        slider.pressed_tint = theme.pressed_tint;
        slider.border_color_focused = Some(theme.focused_border);
        slider.icon = Some(theme.icon_library.icon_handle_check_mark.clone());

        slider
    }

    pub fn current_value(
        mut self,
        current_value: i64,
    ) -> Self {
        self.current_value = current_value;
        self
    }

    pub fn minimum_value(
        mut self,
        minimum_value: i64,
    ) -> Self {
        self.minimum_value = minimum_value;
        self
    }

    pub fn maximum_value(
        mut self,
        maximum_value: i64,
    ) -> Self {
        self.maximum_value = maximum_value;
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

impl<'lifetime> Widget for Slider<'lifetime> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let sense = if self.disabled { Sense::hover() } else { Sense::click_and_drag() };
        let handle_size_float = self.handle_size as f32;
        let allocated_size = vec2(user_interface.available_size().x.min(self.track_width), handle_size_float);
        let (allocated_size_rectangle, mut response) = user_interface.allocate_exact_size(allocated_size, sense);

        // Track (fixed 4px centered).
        let track_height = 8.0;
        let track_rect = Rect::from_min_max(
            pos2(allocated_size_rectangle.left(), allocated_size_rectangle.center().y - track_height / 2.0),
            pos2(allocated_size_rectangle.right(), allocated_size_rectangle.center().y + track_height / 2.0),
        );

        user_interface
            .painter()
            .rect_filled(track_rect, CornerRadius::ZERO, self.background_color_track);

        StateLayer {
            bounds_min: track_rect.min,
            bounds_max: track_rect.max,
            enabled: !self.disabled,
            pressed: response.is_pointer_button_down_on(),
            has_hover: response.hovered(),
            has_focus: response.has_focus(),
            corner_radius: CornerRadius::same(2),
            border_width: 1.0,
            hover_color: self.hover_tint,
            pressed_color: self.pressed_tint,
            border_color: self.border_color,
            border_color_focused: self.border_color_focused.unwrap_or(self.border_color),
        }
        .ui(user_interface);

        // Create circular handle.
        let range = (self.maximum_value - self.minimum_value).max(1);
        let progress = ((self.current_value - self.minimum_value) as f32 / range as f32).clamp(0.0, 1.0);

        let handle_radius = self.handle_size / 2;
        let handle_radius_float = handle_radius as f32;
        let handle_x = allocated_size_rectangle.left() + progress * (allocated_size_rectangle.width() - handle_size_float) + handle_radius_float;
        let handle_center = pos2(handle_x, allocated_size_rectangle.center().y);

        user_interface
            .painter()
            .circle_filled(handle_center, handle_radius_float, self.background_color_handle);

        StateLayer {
            bounds_min: handle_center - vec2(handle_radius_float, handle_radius_float),
            bounds_max: handle_center + vec2(handle_radius_float, handle_radius_float),
            enabled: !self.disabled,
            pressed: response.is_pointer_button_down_on(),
            has_hover: response.hovered(),
            has_focus: response.has_focus(),
            corner_radius: CornerRadius::same(handle_radius),
            border_width: 1.0,
            hover_color: self.hover_tint,
            pressed_color: self.pressed_tint,
            border_color: self.border_color,
            border_color_focused: self.border_color_focused.unwrap_or(self.border_color),
        }
        .ui(user_interface);

        // Interaction.
        if response.dragged() {
            if let Some(mouse_x) = user_interface.input(|i| i.pointer.hover_pos().map(|p| p.x)) {
                let normalized = ((mouse_x - allocated_size_rectangle.left()) / allocated_size_rectangle.width()).clamp(0.0, 1.0);
                let new_value = self.minimum_value + (normalized * range as f32).round() as i64;

                response = response.union(user_interface.interact(track_rect, response.id, Sense::drag()));
                user_interface.memory_mut(|memory| memory.data.insert_temp(response.id, new_value));
            }
        }

        if !self.tooltip_text.is_empty() {
            response = response.on_hover_text(self.tooltip_text);
        }

        if response.clicked() {
            if let Some(sound) = self.click_sound {
                println!("JIRA: Play sound: {}", sound);
            }
        }

        response
    }
}
