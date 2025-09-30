use crate::app_context::AppContext;
use eframe::egui::{Response, Sense, Ui, Widget};
use std::rc::Rc;

#[derive(Clone)]
pub struct SettingsView {
    app_context: Rc<AppContext>,
}

impl SettingsView {
    pub fn new(app_context: Rc<AppContext>) -> Self {
        Self { app_context }
    }
}

impl Widget for SettingsView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (available_size_rect, response) = user_interface.allocate_exact_size(user_interface.available_size(), Sense::empty());

        response
    }
}
