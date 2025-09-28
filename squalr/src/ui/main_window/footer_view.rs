use crate::ui::theme::Theme;
use eframe::egui::{Context, Response, Sense, Ui, Widget};
use epaint::{CornerRadius, vec2};
use std::rc::Rc;

#[derive(Clone)]
pub struct FooterView {
    _context: Context,
    theme: Rc<Theme>,
    corner_radius: CornerRadius,
    height: f32,
}

impl FooterView {
    pub fn new(
        context: Context,
        theme: Rc<Theme>,
        corner_radius: CornerRadius,
        height: f32,
    ) -> Self {
        Self {
            _context: context,
            theme,
            corner_radius,
            height,
        }
    }
}

impl Widget for FooterView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (rect, response) = user_interface.allocate_exact_size(vec2(user_interface.available_size().x, self.height), Sense::empty());

        // Background.
        user_interface.painter().rect_filled(
            rect,
            CornerRadius {
                nw: 0,
                ne: 0,
                sw: self.corner_radius.sw,
                se: self.corner_radius.se,
            },
            self.theme.border_blue,
        );

        response
    }
}
