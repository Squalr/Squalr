use eframe::egui::{Color32, CornerRadius, Stroke, Ui};
use epaint::{Rect, StrokeKind};

#[derive(Default)]
pub struct StateLayer {
    pub enabled: bool,
    pub pressed: bool,
    pub has_hover: bool,
    pub has_focus: bool,

    pub corner_radius: u8,
    pub border_width: f32,

    pub hover_color: Color32,
    pub pressed_color: Color32,
    pub border_color: Color32,
    pub border_color_focused: Color32,
}

impl StateLayer {
    pub fn paint(
        self,
        user_interface: &Ui,
        rect: Rect,
    ) {
        if !user_interface.is_rect_visible(rect) {
            return;
        }

        // Background color depending on state (pressed > hover > default)
        let fill = if self.enabled && self.pressed {
            self.pressed_color
        } else if self.enabled && self.has_hover {
            self.hover_color
        } else {
            Color32::TRANSPARENT
        };

        // Background
        user_interface
            .painter()
            .rect_filled(rect, CornerRadius::same(self.corner_radius), fill);

        // Border
        if self.border_width > 0.0 {
            let border_color = if self.has_focus { self.border_color_focused } else { self.border_color };

            user_interface.painter().rect_stroke(
                rect,
                CornerRadius::same(self.corner_radius),
                Stroke::new(self.border_width, border_color),
                StrokeKind::Inside,
            );
        }
    }
}
