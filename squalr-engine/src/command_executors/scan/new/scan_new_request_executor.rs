use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan::new::scan_new_request::ScanNewRequest;
use squalr_engine_api::commands::scan::new::scan_new_response::ScanNewResponse;
use squalr_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use squalr_engine_memory::memory_queryer::memory_queryer::MemoryQueryer;
use squalr_engine_memory::memory_queryer::page_retrieval_mode::PageRetrievalMode;
use squalr_engine_scanning::snapshots::snapshot_region::SnapshotRegion;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ScanNewRequest {
    type ResponseType = ScanNewResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        let opened_process_info = engine_privileged_state
            .get_process_manager()
            .get_opened_process();
        let opened_process_info = match opened_process_info {
            Some(opened_process_info) => opened_process_info,
            None => {
                log::error!("No opened process, cannot start new scan.");

                return ScanNewResponse {};
            }
        };

        let snapshot = engine_privileged_state.get_snapshot();
        let mut snapshot = match snapshot.write() {
            Ok(guard) => guard,
            Err(err) => {
                log::error!("Failed to acquire write lock on snapshot: {}", err);

                return ScanNewResponse {};
            }
        };

        let snapshot_scan_result_freeze_list = engine_privileged_state.get_snapshot_scan_result_freeze_list();

        // Best-effort to clear the freeze list.
        match snapshot_scan_result_freeze_list.read() {
            Ok(snapshot_scan_result_freeze_list) => {
                snapshot_scan_result_freeze_list.clear();
            }
            Err(err) => {
                log::error!("Failed to acquire write lock on snapshot: {}", err);
            }
        }

        // Query all memory pages for the process from the OS.
        let memory_pages = MemoryQueryer::get_memory_page_bounds(&opened_process_info, PageRetrievalMode::FromSettings);

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
