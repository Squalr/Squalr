use crate::ui::theme::Theme;
use eframe::egui::{Context, Response, Sense, Ui};
use epaint::{CornerRadius, vec2};
use std::rc::Rc;

#[derive(Clone)]
pub struct DockedWindowTitleBarView {
    context: Context,
    theme: Rc<Theme>,
    height: f32,
    title: String,
}

impl DockedWindowTitleBarView {
    pub fn new(
        context: Context,
        theme: Rc<Theme>,
        title: String,
    ) -> Self {
        let height = 32.0;

        Self { context, theme, height, title }
    }
}

impl eframe::egui::Widget for DockedWindowTitleBarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (rect, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), self.height), Sense::empty());

        user_interface
            .painter()
            .rect_filled(rect, CornerRadius::ZERO, self.theme.background_primary);

        response
    }
}
