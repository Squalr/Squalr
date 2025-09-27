use crate::ui::theme::Theme;
use eframe::egui::Widget;
use eframe::egui::{Context, Response, Sense, Ui};
use epaint::{CornerRadius, vec2};
use std::rc::Rc;

#[derive(Clone)]
pub struct ToolbarView {
    pub context: Context,
    pub theme: Rc<Theme>,
    pub height: f32,
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
