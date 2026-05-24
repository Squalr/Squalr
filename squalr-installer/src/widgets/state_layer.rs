use eframe::egui::{Color32, CornerRadius, Response, Sense, Stroke, Ui, Widget};
use epaint::{Pos2, Rect, StrokeKind};

#[derive(Default)]
pub(crate) struct StateLayer {
    pub(crate) bounds_min: Pos2,
    pub(crate) bounds_max: Pos2,
    pub(crate) enabled: bool,
    pub(crate) pressed: bool,
    pub(crate) has_hover: bool,
    pub(crate) has_focus: bool,
    pub(crate) corner_radius: CornerRadius,
    pub(crate) border_width: f32,
    pub(crate) hover_color: Color32,
    pub(crate) pressed_color: Color32,
    pub(crate) border_color: Color32,
    pub(crate) border_color_focused: Color32,
}

impl Widget for StateLayer {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let bounds_rectangle = Rect {
            min: self.bounds_min,
            max: self.bounds_max,
        };
        let response = user_interface.interact(bounds_rectangle, user_interface.id().with("state_layer"), Sense::hover());

        if !user_interface.is_rect_visible(bounds_rectangle) {
            return response;
        }

        let fill_color = if self.enabled && self.pressed {
            self.pressed_color
        } else if self.enabled && self.has_hover {
            self.hover_color
        } else {
            Color32::TRANSPARENT
        };

        user_interface
            .painter()
            .rect_filled(bounds_rectangle, self.corner_radius, fill_color);

        if self.border_width > 0.0 {
            let border_color = if self.has_focus { self.border_color_focused } else { self.border_color };

            user_interface.painter().rect_stroke(
                bounds_rectangle,
                self.corner_radius,
                Stroke::new(self.border_width, border_color),
                StrokeKind::Inside,
            );
        }

        response
    }
}
