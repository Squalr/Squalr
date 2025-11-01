use crate::app_context::AppContext;
use eframe::egui::{Response, Sense, Ui, Widget};
use std::sync::Arc;

#[derive(Clone)]
pub struct ProjectExplorerView {
    app_context: Arc<AppContext>,
}

impl ProjectExplorerView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        Self { app_context }
    }
}

impl Widget for ProjectExplorerView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(user_interface.available_size(), Sense::empty());

        response
    }
}
