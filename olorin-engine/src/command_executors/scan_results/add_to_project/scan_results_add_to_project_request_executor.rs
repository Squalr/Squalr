use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use olorin_engine_api::commands::scan_results::add_to_project::scan_results_add_to_project_request::ScanResultsAddToProjectRequest;
use olorin_engine_api::commands::scan_results::add_to_project::scan_results_add_to_project_response::ScanResultsAddToProjectResponse;
use olorin_engine_api::structures::projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress;
use olorin_engine_memory::memory_queryer::memory_queryer::MemoryQueryer;
use olorin_engine_memory::memory_queryer::memory_queryer_trait::IMemoryQueryer;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ScanResultsAddToProjectRequest {
    type ResponseType = ScanResultsAddToProjectResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        let symbol_registry = engine_privileged_state.get_registries().get_symbol_registry();
        let symbol_registry_guard = match symbol_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on SymbolRegistry: {}", error);

                return ScanResultsAddToProjectResponse::default();
            }
        };
        let snapshot = engine_privileged_state.get_snapshot();
        let snapshot_guard = match snapshot.read() {
            Ok(snapshot) => snapshot,
            Err(error) => {
                log::error!("Failed to acquire read lock on Snapshot: {}", error);

                return ScanResultsAddToProjectResponse::default();
            }
        };
        let project_manager = engine_privileged_state.get_project_manager();
        let mut project_changed = false;
        let modules = if let Some(opened_process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        {
            MemoryQueryer::get_instance().get_modules(&opened_process_info)
        } else {
            vec![]
        };

        match project_manager.get_opened_project().write() {
            Ok(mut opened_project) => {
                if let Some(project) = opened_project.as_mut() {
                    for scan_result_ref in &self.scan_result_refs {
                        if let Some(scan_result) = snapshot_guard.get_scan_result(&symbol_registry, scan_result_ref.get_scan_result_index()) {
                            let data_type_ref = scan_result.get_data_type_ref();

                            if let Some(data_value) = symbol_registry_guard.get_default_value(data_type_ref) {
                                let address = scan_result.get_address();
                                let path = project.get_project_root().get_path().join("Address");
                                let description = String::new();

                                let mut module_offset = address;
                                let mut module_name = String::default();

                                // Check whether this scan result belongs to a module (ie check if the address is static).
                                if let Some((found_module_name, address)) = MemoryQueryer::get_instance().address_to_module(address, &modules) {
                                    module_name = found_module_name;
                                    module_offset = address;
                                }

                                let address_item = ProjectItemTypeAddress::new_project_item(&path, module_offset, &module_name, &description, data_value);

                                // Add to project root.
                                project.get_project_root_mut().append_child(address_item);
                                project_changed = true;
                            } else {
                                log::error!("Error adding scan result, unable to get default value. The data type may no longer be registered.");
                            }
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

        ScanResultsAddToProjectResponse::default()
    }
}
