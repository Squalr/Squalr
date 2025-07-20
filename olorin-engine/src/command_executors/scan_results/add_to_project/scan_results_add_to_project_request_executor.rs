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
        let data_type_registry = engine_privileged_state.get_data_type_registry();
        let data_type_registry_guard = match data_type_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on DataTypeRegistry: {}", error);

                return ScanResultsAddToProjectResponse {};
            }
        };
        let project_manager = engine_privileged_state.get_project_manager();
        let mut project_changed = false;

        match project_manager.get_opened_project().write() {
            Ok(mut opened_project) => {
                if let Some(project) = opened_project.as_mut() {
                    for scan_result in &self.scan_results {
                        let data_type_ref = scan_result.get_data_type_ref();
                        if let Some(data_value) = data_type_registry_guard.get_default_value(data_type_ref) {
                            let address = scan_result.get_address();
                            let path = project.get_project_root().get_path().join("Address");
                            let module = scan_result.get_module();
                            let description = String::new();
                            let address_item = ProjectItemTypeAddress::new_project_item(&path, address, module, &description, data_value);

                            // Add to project root.
                            project.get_project_root_mut().append_child(address_item);
                            project_changed = true;
                        } else {
                            log::error!("Error adding scan result, unable to get default value. The data type may no longer be registered.");
                        }
                    }

                    if let Err(error) = project.save(true) {
                        log::error!("Failed to save project after adding scan results: {}", error);
                    }
                } else {
                    log::warn!("Unable to add scan results, no opened project.");
                }
            }
            Err(error) => {
                log::error!("Error opening project: {}", error);
            }
        }

        if project_changed {
            project_manager.notify_project_items_changed();
        }

        ScanResultsAddToProjectResponse {}
    }
}
