use crate::{app_context::AppContext, views::struct_viewer::view_data::struct_viewer_view_data::StructViewerViewData};
use eframe::egui::{Response, Sense, Ui, Widget};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use std::sync::Arc;

#[derive(Clone)]
pub struct StructViewerView {
    app_context: Arc<AppContext>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
}

impl StructViewerView {
    pub const WINDOW_ID: &'static str = "window_struct_viewer";

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let struct_viewer_view_data = app_context
            .dependency_container
            .register(StructViewerViewData::new());

        Self {
            app_context,
            struct_viewer_view_data,
        }
    }
}

impl Widget for StructViewerView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(user_interface.available_size(), Sense::empty());

        response
    }
}
