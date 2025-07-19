use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use olorin_engine_api::commands::scan_results::add_to_project::scan_results_add_to_project_request::ScanResultsAddToProjectRequest;
use olorin_engine_api::commands::scan_results::add_to_project::scan_results_add_to_project_response::ScanResultsAddToProjectResponse;
use olorin_engine_api::structures::projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ScanResultsAddToProjectRequest {
    type ResponseType = ScanResultsAddToProjectResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        let project_manager = engine_privileged_state.get_project_manager();
        let opened_project_lock = project_manager.get_opened_project();
        let mut opened_project_guard = opened_project_lock.write().unwrap();

        if let Some(project) = opened_project_guard.as_mut() {
            for scan_result in &self.scan_results {
                let address = scan_result.get_address();
                let data_type = scan_result.get_data_type().clone();
                let data_value = data_type.get_default_value().unwrap_or_default();
                let path = project.get_project_root().get_path().join("Address");
                let description = String::new();
                let address_item = ProjectItemTypeAddress::new_project_item(&path, address, &description, data_value);

                // Add to project root.
                project.get_project_root_mut().append_child(address_item);
            }

            if let Err(err) = project.save(true) {
                log::error!("Failed to save project after adding scan results: {}", err);
            }
        } else {
            log::warn!("Unable to add scan results, no opened project.");
        }

        ScanResultsAddToProjectResponse {}
    }
}
