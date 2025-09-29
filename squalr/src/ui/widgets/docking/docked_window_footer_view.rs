use crate::ui::theme::Theme;
use eframe::egui::{Context, Response, Sense, Ui, Widget};
use epaint::{CornerRadius, vec2};
use std::rc::Rc;

#[derive(Clone)]
pub struct DockedWindowFooterView {
    _context: Context,
    theme: Rc<Theme>,
    height: f32,
}

impl DockedWindowFooterView {
    pub fn new(
        context: Context,
        theme: Rc<Theme>,
    ) -> Self {
        let height = 28.0;

        Self {
            _context: context,
            theme,
            height,
        }
    }

    pub fn get_height(&self) -> f32 {
        self.height
    }
}

impl Widget for DockedWindowFooterView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (rect, response) = user_interface.allocate_exact_size(vec2(user_interface.available_size().x, self.height), Sense::empty());

        // Background.
        user_interface
            .painter()
            .rect_filled(rect, CornerRadius::ZERO, self.theme.border_blue);

        response
    }
}
