use crate::{app_context::AppContext, views::process_selector::process_selector_view_data::ProcessSelectorViewData};
use eframe::egui::{Response, Sense, Ui, Widget};
use squalr_engine_api::{
    commands::{engine_command_request::EngineCommandRequest, process::list::process_list_request::ProcessListRequest},
    dependency_injection::dependency::Dependency,
    engine::engine_execution_context::EngineExecutionContext,
};
use std::sync::Arc;

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

    fn refresh_full_process_list(
        engine_execution_context: Arc<EngineExecutionContext>,
        process_selector_view_data: Dependency<ProcessSelectorViewData>,
    ) {
        let list_windowed_processes_request = ProcessListRequest {
            require_windowed: false,
            search_name: None,
            match_case: false,
            limit: None,
            fetch_icons: true,
        };
        let engine_execution_context = engine_execution_context.clone();

        list_windowed_processes_request.send(&engine_execution_context, move |process_list_response| {
            let mut process_selector_view_data = match process_selector_view_data.write() {
                Ok(process_selector_view_data) => process_selector_view_data,
                Err(error) => {
                    log::error!("Failed to access process selector view data for updating windowed process list: {}", error);
                    return;
                }
            };

            process_selector_view_data.set_full_process_list(process_list_response.processes);
        });
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
