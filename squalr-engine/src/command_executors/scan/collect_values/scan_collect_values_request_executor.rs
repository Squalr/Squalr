use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::command_executors::scan::scan_initializer::{ensure_snapshot_regions_for_scan, initialize_snapshot_scan_results_if_empty};
use crate::command_executors::scan::scan_results_metadata_collector::collect_scan_results_metadata;
use crate::command_executors::scan::snapshot_value_collector::SnapshotValueCollector;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan::collect_values::scan_collect_values_request::ScanCollectValuesRequest;
use squalr_engine_api::commands::scan::collect_values::scan_collect_values_response::ScanCollectValuesResponse;
use squalr_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
use squalr_engine_session::settings::scan_settings_store::ScanSettingsStore;
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
            let memory_alignment = ScanSettingsStore::get_memory_alignment().unwrap_or(MemoryAlignment::Alignment1);
            let did_initialize_new_scan = match engine_privileged_state.read_symbol_registry(|symbol_registry| match snapshot.write() {
                Ok(mut snapshot_guard) => Some({
                    ensure_snapshot_regions_for_scan(engine_privileged_state, &process_info, &mut snapshot_guard);

                    if snapshot_guard.get_region_count() == 0 {
                        false
                    } else {
                        initialize_snapshot_scan_results_if_empty(&mut snapshot_guard, symbol_registry, &self.data_type_refs, memory_alignment)
                    }
                }),
                Err(error) => {
                    log::error!("Failed to acquire write lock on snapshot before value collection: {}", error);

                    None
                }
            }) {
                Some(did_initialize_new_scan) => did_initialize_new_scan,
                None => return ScanCollectValuesResponse::default(),
            };

            if did_initialize_new_scan {
                let freeze_list_registry = engine_privileged_state.get_freeze_list_registry();

                if let Ok(mut freeze_list_registry_guard) = freeze_list_registry.write() {
                    freeze_list_registry_guard.clear();
                } else {
                    log::error!("Failed to acquire write lock on FreezeListRegistry before value collection.");
                }
            }

            SnapshotValueCollector::collect_values(
                process_info.clone(),
                snapshot,
                engine_privileged_state.get_os_providers().memory_read.clone(),
                true,
            );
            engine_privileged_state.emit_event(ScanResultsUpdatedEvent {
                is_new_scan: did_initialize_new_scan,
            });

            ScanCollectValuesResponse {
                success: true,
                scan_results_metadata: collect_scan_results_metadata(engine_privileged_state),
            }
        } else {
            log::error!("No opened process");
            ScanCollectValuesResponse::default()
        }
    }
}
