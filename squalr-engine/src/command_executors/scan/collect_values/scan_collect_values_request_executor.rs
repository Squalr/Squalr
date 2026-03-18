use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::command_executors::scan::scan_initializer::{ensure_snapshot_regions_for_scan, initialize_snapshot_scan_results_if_empty};
use crate::command_executors::scan::scan_results_metadata_collector::collect_scan_results_metadata;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan::collect_values::scan_collect_values_request::ScanCollectValuesRequest;
use squalr_engine_api::commands::scan::collect_values::scan_collect_values_response::ScanCollectValuesResponse;
use squalr_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
use squalr_engine_scanning::scan_settings_config::ScanSettingsConfig;
use squalr_engine_scanning::scanners::scan_execution_context::ScanExecutionContext;
use squalr_engine_scanning::scanners::value_collector_task::ValueCollector;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for ScanCollectValuesRequest {
    type ResponseType = ScanCollectValuesResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        if let Some(process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        {
            let snapshot = engine_privileged_state.get_snapshot();
            let memory_alignment = ScanSettingsConfig::get_memory_alignment().unwrap_or(MemoryAlignment::Alignment1);
            let did_initialize_new_scan = match snapshot.write() {
                Ok(mut snapshot_guard) => {
                    ensure_snapshot_regions_for_scan(engine_privileged_state, &process_info, &mut snapshot_guard);

                    if snapshot_guard.get_region_count() == 0 {
                        false
                    } else {
                        initialize_snapshot_scan_results_if_empty(&mut snapshot_guard, &self.data_type_refs, memory_alignment)
                    }
                }
                Err(error) => {
                    log::error!("Failed to acquire write lock on snapshot before value collection: {}", error);

                    return ScanCollectValuesResponse::default();
                }
            };

            if did_initialize_new_scan {
                let freeze_list_registry = engine_privileged_state.get_freeze_list_registry();

                if let Ok(mut freeze_list_registry_guard) = freeze_list_registry.write() {
                    freeze_list_registry_guard.clear();
                } else {
                    log::error!("Failed to acquire write lock on FreezeListRegistry before value collection.");
                }
            }

            let memory_read_provider = engine_privileged_state.get_os_providers().memory_read.clone();
            let scan_execution_context = ScanExecutionContext::new(
                None,
                None,
                Some(Arc::new(move |opened_process_info, address, values| {
                    memory_read_provider.read_bytes(opened_process_info, address, values)
                })),
            );
            ValueCollector::collect_values(process_info.clone(), snapshot, true, &scan_execution_context);
            engine_privileged_state.emit_event(ScanResultsUpdatedEvent {
                is_new_scan: did_initialize_new_scan,
            });

            ScanCollectValuesResponse {
                scan_results_metadata: collect_scan_results_metadata(engine_privileged_state),
            }
        } else {
            log::error!("No opened process");
            ScanCollectValuesResponse::default()
        }
    }
}
