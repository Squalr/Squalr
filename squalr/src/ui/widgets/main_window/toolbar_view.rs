use crate::ui::theme::Theme;
use eframe::egui::Widget;
use eframe::egui::{Context, Response, Sense, Ui};
use epaint::{CornerRadius, vec2};
use std::rc::Rc;

#[derive(Clone)]
pub struct ToolbarView {
    _context: Context,
    theme: Rc<Theme>,
    height: f32,
}

impl ToolbarView {
    pub fn new(
        context: Context,
        theme: Rc<Theme>,
        height: f32,
    ) -> Self {
        Self {
            _context: context,
            theme,
            height,
        }
    }
}

impl Widget for ToolbarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (rect, response) = user_interface.allocate_exact_size(vec2(user_interface.available_size().x, self.height), Sense::empty());

        // Background.
        user_interface
            .painter()
            .rect_filled(rect, CornerRadius::ZERO, self.theme.background_primary);

        response
    }
}
