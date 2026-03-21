use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::command_executors::snapshot_region_builder::merge_memory_regions_into_snapshot_regions;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::pointer_scan::validate::pointer_scan_validate_request::PointerScanValidateRequest;
use squalr_engine_api::commands::pointer_scan::validate::pointer_scan_validate_response::PointerScanValidateResponse;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use squalr_engine_scanning::pointer_scans::pointer_scan_validator::PointerScanValidator;
use squalr_engine_scanning::scanners::scan_execution_context::ScanExecutionContext;
use squalr_engine_scanning::scanners::value_collector_task::ValueCollector;
use squalr_engine_session::os::PageRetrievalMode;
use std::sync::{Arc, RwLock};

impl PrivilegedCommandRequestExecutor for PointerScanValidateRequest {
    type ResponseType = PointerScanValidateResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let Some(process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        else {
            return PointerScanValidateResponse {
                validation_performed: false,
                validation_target_address: None,
                pruned_node_count: 0,
                status_message: "No opened process is available for pointer validation.".to_string(),
                pointer_scan_summary: None,
            };
        };

        let pointer_scan_session = match engine_privileged_state.get_pointer_scan_session().read() {
            Ok(pointer_scan_session_guard) => match pointer_scan_session_guard.as_ref() {
                Some(pointer_scan_session) => pointer_scan_session.clone(),
                None => {
                    return PointerScanValidateResponse {
                        validation_performed: false,
                        validation_target_address: None,
                        pruned_node_count: 0,
                        status_message: "No active pointer scan session is available.".to_string(),
                        pointer_scan_summary: None,
                    };
                }
            },
            Err(error) => {
                log::error!("Failed to acquire read lock on pointer scan session store: {}", error);

                return PointerScanValidateResponse {
                    validation_performed: false,
                    validation_target_address: None,
                    pruned_node_count: 0,
                    status_message: "Failed to access the active pointer scan session.".to_string(),
                    pointer_scan_summary: None,
                };
            }
        };

        if pointer_scan_session.get_session_id() != self.session_id {
            return PointerScanValidateResponse {
                validation_performed: false,
                validation_target_address: None,
                pruned_node_count: 0,
                status_message: format!("Pointer scan session {} was not found.", self.session_id),
                pointer_scan_summary: None,
            };
        }

        let symbol_registry = engine_privileged_state.get_symbol_registry();
        let symbol_registry_guard = match symbol_registry.read() {
            Ok(symbol_registry_guard) => symbol_registry_guard,
            Err(error) => {
                log::error!("Failed to acquire read lock on SymbolRegistry: {}", error);

                return PointerScanValidateResponse {
                    validation_performed: false,
                    validation_target_address: None,
                    pruned_node_count: 0,
                    status_message: "Failed to access the symbol registry for validation.".to_string(),
                    pointer_scan_summary: Some(pointer_scan_session.summarize()),
                };
            }
        };
        let pointer_size = pointer_scan_session.get_pointer_size();
        let validation_target_data_type_ref = pointer_size.to_data_type_ref();
        let validation_target_data_value = match symbol_registry_guard.deanonymize_value_string(&validation_target_data_type_ref, &self.target_address) {
            Ok(validation_target_data_value) => validation_target_data_value,
            Err(error) => {
                log::error!("Failed to deanonymize pointer scan validation target address: {}", error);

                return PointerScanValidateResponse {
                    validation_performed: false,
                    validation_target_address: None,
                    pruned_node_count: 0,
                    status_message: "Failed to parse the validation target address.".to_string(),
                    pointer_scan_summary: Some(pointer_scan_session.summarize()),
                };
            }
        };
        let Some(validation_target_address) = pointer_size.read_address_value(&validation_target_data_value) else {
            return PointerScanValidateResponse {
                validation_performed: false,
                validation_target_address: None,
                pruned_node_count: 0,
                status_message: "Failed to decode the validation target address.".to_string(),
                pointer_scan_summary: Some(pointer_scan_session.summarize()),
            };
        };
        drop(symbol_registry_guard);

        let modules = engine_privileged_state
            .get_os_providers()
            .memory_query
            .get_modules(&process_info);
        let memory_regions = engine_privileged_state
            .get_os_providers()
            .memory_query
            .get_memory_page_bounds(&process_info, PageRetrievalMode::FromUserMode);
        let memory_read_provider = engine_privileged_state.get_os_providers().memory_read.clone();
        let scan_execution_context = ScanExecutionContext::new(
            None,
            None,
            Some(Arc::new(move |opened_process_info, address, values| {
                memory_read_provider.read_bytes(opened_process_info, address, values)
            })),
        );
        let mut validation_snapshot = Snapshot::new();
        validation_snapshot.set_snapshot_regions(merge_memory_regions_into_snapshot_regions(memory_regions));
        let validation_snapshot = Arc::new(RwLock::new(validation_snapshot));

        ValueCollector::collect_values(process_info.clone(), validation_snapshot.clone(), true, &scan_execution_context);

        let validation_snapshot_guard = match validation_snapshot.read() {
            Ok(validation_snapshot_guard) => validation_snapshot_guard,
            Err(error) => {
                log::error!("Failed to acquire read lock on validation snapshot: {}", error);

                return PointerScanValidateResponse {
                    validation_performed: false,
                    validation_target_address: None,
                    pruned_node_count: 0,
                    status_message: "Failed to access the validation snapshot.".to_string(),
                    pointer_scan_summary: Some(pointer_scan_session.summarize()),
                };
            }
        };
        let validated_pointer_scan_session = PointerScanValidator::validate_scan(
            process_info,
            &pointer_scan_session,
            validation_target_address,
            &validation_snapshot_guard,
            &modules,
            &scan_execution_context,
            true,
        );
        let validated_pointer_scan_summary = validated_pointer_scan_session.summarize();
        let pruned_node_count = pointer_scan_session
            .get_total_node_count()
            .saturating_sub(validated_pointer_scan_session.get_total_node_count());

        match engine_privileged_state.get_pointer_scan_session().write() {
            Ok(mut pointer_scan_session_guard) => {
                *pointer_scan_session_guard = Some(validated_pointer_scan_session);
            }
            Err(error) => {
                log::error!("Failed to acquire write lock on pointer scan session store: {}", error);
            }
        }

        PointerScanValidateResponse {
            validation_performed: true,
            validation_target_address: Some(validation_target_address),
            pruned_node_count,
            status_message: format!(
                "Validated pointer scan session {} against 0x{:X}. Pruned {} nodes.",
                self.session_id, validation_target_address, pruned_node_count
            ),
            pointer_scan_summary: Some(validated_pointer_scan_summary),
        }
    }
}
