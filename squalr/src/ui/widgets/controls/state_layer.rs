use eframe::egui::{Color32, CornerRadius, Response, Sense, Stroke, Ui, Widget};
use epaint::{Pos2, Rect, StrokeKind};

#[derive(Default)]
pub struct StateLayer {
    pub bounds_min: Pos2,
    pub bounds_max: Pos2,

    pub enabled: bool,
    pub pressed: bool,
    pub has_hover: bool,
    pub has_focus: bool,

    pub corner_radius: CornerRadius,
    pub border_width: f32,

    pub hover_color: Color32,
    pub pressed_color: Color32,
    pub border_color: Color32,
    pub border_color_focused: Color32,
}

impl Widget for StateLayer {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let bounds_rect = Rect {
            min: self.bounds_min,
            max: self.bounds_max,
        };
        let response = user_interface.interact(bounds_rect, user_interface.id().with("state_layer"), Sense::hover());

        if !user_interface.is_rect_visible(bounds_rect) {
            return response;
        }

        // Select background color depending on state (pressed > hover > default).
        let fill = if self.enabled && self.pressed {
            self.pressed_color
        } else if self.enabled && self.has_hover {
            self.hover_color
        } else {
            Color32::TRANSPARENT
        };

        // Draw the background.
        user_interface
            .painter()
            .rect_filled(bounds_rect, self.corner_radius, fill);

        // Draw the border.
        if self.border_width > 0.0 {
            let border_color = if self.has_focus { self.border_color_focused } else { self.border_color };

            user_interface.painter().rect_stroke(
                bounds_rect,
                self.corner_radius,
                Stroke::new(self.border_width, border_color),
                StrokeKind::Inside,
            );
        }

        response
    }
}
