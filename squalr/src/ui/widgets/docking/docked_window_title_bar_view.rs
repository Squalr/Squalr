use crate::ui::theme::Theme;
use eframe::egui::{Context, Response, Sense, Ui};
use epaint::{CornerRadius, vec2};
use std::rc::Rc;

#[derive(Clone)]
pub struct DockedWindowTitleBarView {
    context: Context,
    theme: Rc<Theme>,
    corner_radius: CornerRadius,
    height: f32,
    title: String,
}

impl DockedWindowTitleBarView {
    pub fn new(
        context: Context,
        theme: Rc<Theme>,
        corner_radius: CornerRadius,
        height: f32,
        title: String,
    ) -> Self {
        Self {
            context,
            theme,
            corner_radius,
            height,
            title,
        }
    }
}

impl eframe::egui::Widget for DockedWindowTitleBarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (rect, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), self.height), Sense::empty());

        user_interface.painter().rect_filled(
            rect,
            CornerRadius {
                nw: self.corner_radius.nw,
                ne: self.corner_radius.ne,
                sw: 0,
                se: 0,
            },
            self.theme.background_primary,
        );

        response
    }
}
