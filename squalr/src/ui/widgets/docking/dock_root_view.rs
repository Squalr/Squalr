use crate::ui::theme::Theme;
use eframe::egui::{Context, Response, Sense, Ui, Widget};
use epaint::CornerRadius;
use std::rc::Rc;

#[derive(Clone)]
pub struct DockRootView {
    _context: Context,
    theme: Rc<Theme>,
}

impl DockRootView {
    pub fn new(
        context: Context,
        theme: Rc<Theme>,
    ) -> Self {
        Self { _context: context, theme }
    }
}

impl Widget for DockRootView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (rect, response) = user_interface.allocate_exact_size(user_interface.available_size(), Sense::empty());

        // Background.
        user_interface
            .painter()
            .rect_filled(rect, CornerRadius::ZERO, self.theme.hex_green);

        response
    }
}
