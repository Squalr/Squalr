use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan_results::freeze::scan_results_freeze_request::ScanResultsFreezeRequest;
use squalr_engine_api::commands::scan_results::set_property::scan_results_set_property_request::ScanResultsSetPropertyRequest;
use squalr_engine_api::commands::scan_results::set_property::scan_results_set_property_response::ScanResultsSetPropertyResponse;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::structures::data_types::built_in_types::bool32::data_type_bool32::DataTypeBool32;
use squalr_engine_api::structures::data_types::data_type::DataType;
use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for ScanResultsSetPropertyRequest {
    type ResponseType = ScanResultsSetPropertyResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let snapshot = engine_privileged_state.get_snapshot();
        let snapshot_guard = match snapshot.read() {
            Ok(snapshot) => snapshot,
            Err(error) => {
                log::error!("Failed to acquire read lock on Snapshot: {}", error);

                return ScanResultsSetPropertyResponse::default();
            }
        };
        let symbol_registry = SymbolRegistry::get_instance();
        let os_providers = engine_privileged_state.get_os_providers();

        match self.field_namespace.as_str() {
            ScanResult::PROPERTY_NAME_VALUE => {
                for scan_result_ref in &self.scan_result_refs {
                    if let Some(scan_result) = snapshot_guard.get_scan_result(scan_result_ref.get_scan_result_global_index()) {
                        if let Ok(data_value) = symbol_registry.deanonymize_value_string(scan_result.get_data_type_ref(), &self.anonymous_value_string) {
                            let value_bytes = data_value.get_value_bytes();
                            let address = scan_result.get_address();
                            if let Some(opened_process_info) = engine_privileged_state
                                .get_process_manager()
                                .get_opened_process()
                            {
                                // Best-effort attempt to write the property bytes.
                                let _ = os_providers
                                    .memory_write
                                    .write_bytes(&opened_process_info, address, &value_bytes);
                            }
                        }
                    }
                }
            }
            ScanResult::PROPERTY_NAME_IS_FROZEN => {
                let data_type = DataTypeBool32 {};
                if let Ok(data_value) = data_type.deanonymize_value_string(&self.anonymous_value_string) {
                    let is_frozen = data_value.get_value_bytes().iter().any(|&byte| byte != 0);

                    // Fire an internal request to freeze.
                    let scan_results_freeze_request = ScanResultsFreezeRequest {
                        scan_result_refs: self.scan_result_refs.clone(),
                        is_frozen,
                    };

                    scan_results_freeze_request.execute(engine_privileged_state);
                }
            }
            ScanResult::PROPERTY_NAME_ADDRESS | ScanResult::PROPERTY_NAME_MODULE | ScanResult::PROPERTY_NAME_MODULE_OFFSET => {
                log::warn!("Cannot set read-only property {}", self.field_namespace);
            }
            _ => {
                log::warn!("Attempted to set unsupported property on scan result.");
            }
        }

        ScanResultsSetPropertyResponse::default()
    }
}
