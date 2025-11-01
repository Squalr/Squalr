use crate::app_context::AppContext;
use eframe::egui::{Response, Sense, Ui, Widget};
use epaint::{CornerRadius, vec2};
use std::sync::Arc;

#[derive(Clone)]
pub struct MainFooterView {
    app_context: Arc<AppContext>,
    corner_radius: CornerRadius,
    height: f32,
}

impl MainFooterView {
    pub fn new(
        app_context: Arc<AppContext>,
        corner_radius: CornerRadius,
        height: f32,
    ) -> Self {
        Self {
            app_context,
            corner_radius,
            height,
        }
    }

    pub fn get_height(&self) -> f32 {
        self.height
    }
}

impl Widget for MainFooterView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (available_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_size().x, self.height), Sense::empty());
        let theme = &self.app_context.theme;

        // Background.
        user_interface.painter().rect_filled(
            available_size_rectangle,
            CornerRadius {
                nw: 0,
                ne: 0,
                sw: self.corner_radius.sw,
                se: self.corner_radius.se,
            },
            theme.border_blue,
        );

        response
    }
}
