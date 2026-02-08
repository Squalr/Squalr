use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan::new::scan_new_request::ScanNewRequest;
use squalr_engine_api::commands::scan::new::scan_new_response::ScanNewResponse;
use squalr_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_memory::memory_queryer::page_retrieval_mode::PageRetrievalMode;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for ScanNewRequest {
    type ResponseType = ScanNewResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let freeze_list_registry = engine_privileged_state.get_freeze_list_registry();
        let mut freeze_list_registry_guard = match freeze_list_registry.write() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire write lock on FreezeListRegistry: {}", error);

                return ScanNewResponse::default();
            }
        };
        let opened_process_info = engine_privileged_state
            .get_process_manager()
            .get_opened_process();
        let opened_process_info = match opened_process_info {
            Some(opened_process_info) => opened_process_info,
            None => {
                log::error!("Cannot start new scan, no opened process.");

                return ScanNewResponse::default();
            }
        };

        let snapshot = engine_privileged_state.get_snapshot();
        let mut snapshot = match snapshot.write() {
            Ok(guard) => guard,
            Err(error) => {
                log::error!("Failed to acquire write lock on snapshot: {}", error);

                return ScanNewResponse::default();
            }
        };

        // Best-effort to clear the freeze list.
        freeze_list_registry_guard.clear();

        // Query all memory pages for the process from the OS.
        let memory_pages = engine_privileged_state
            .get_os_providers()
            .memory_query
            .get_memory_page_bounds(&opened_process_info, PageRetrievalMode::FromSettings);

        // Attempt to merge any adjacent regions. This drastically simplifies the scanning process by eliminating edge case handling.
        // Additionally, we must track the page boundaries at which the merge took place.
        // Doing this allows us to ensure that we do not try to read memory across a page boundary later when collecting values.
        // This prevents issues where one page may deallocate, which would otherwise cause the read for an adjacent page to fail!
        let mut merged_snapshot_regions = vec![];
        let mut page_boundaries = vec![];
        let mut iter = memory_pages.into_iter();
        let current_region = iter.next();

        if let Some(mut current_region) = current_region {
            loop {
                let Some(region) = iter.next() else {
                    break;
                };

                if current_region.get_end_address() == region.get_base_address() {
                    current_region.set_end_address(region.get_end_address());
                    page_boundaries.push(region.get_base_address());
                } else {
                    merged_snapshot_regions.push(SnapshotRegion::new(current_region, std::mem::take(&mut page_boundaries)));
                    current_region = region;
                }
            }

            // Push the last region.
            merged_snapshot_regions.push(SnapshotRegion::new(current_region, page_boundaries));

            // Update snapshot with new merged regions.
            snapshot.set_snapshot_regions(merged_snapshot_regions);

            engine_privileged_state.emit_event(ScanResultsUpdatedEvent { is_new_scan: true });
        }

        ScanNewResponse {}
    }
}
