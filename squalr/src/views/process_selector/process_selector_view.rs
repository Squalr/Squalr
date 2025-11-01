use crate::{app_context::AppContext, views::process_selector::process_selector_view_data::ProcessSelectorViewData};
use eframe::egui::{Response, Sense, Ui, Widget};
use epaint::mutex::RwLock;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use std::{rc::Rc, sync::Arc};

#[derive(Clone)]
pub struct ProcessSelectorView {
    app_context: Arc<AppContext>,
    process_selector_view_data: Dependency<ProcessSelectorViewData>,
}

impl ProcessSelectorView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let process_selector_view_data = app_context
            .dependency_container
            .register(ProcessSelectorViewData::new());

        Self {
            app_context,
            process_selector_view_data,
        }
    }
}

impl Widget for ProcessSelectorView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (available_size_rectangle, response) = user_interface.allocate_exact_size(user_interface.available_size(), Sense::empty());

        response
    }
}
