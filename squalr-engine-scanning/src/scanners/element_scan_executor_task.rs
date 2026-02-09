use crate::scanners::element_scan_dispatcher::ElementScanDispatcher;
use crate::scanners::scan_execution_context::ScanExecutionContext;
use crate::scanners::snapshot_region_memory_reader::SnapshotRegionMemoryReader;
use crate::scanners::value_collector_task::ValueCollector;
use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use squalr_engine_api::conversions::storage_size_conversions::StorageSizeConversions;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::results::snapshot_region_scan_results::SnapshotRegionScanResults;
use squalr_engine_api::structures::scanning::memory_read_mode::MemoryReadMode;
use squalr_engine_api::structures::scanning::plans::element_scan::element_scan_plan::ElementScanPlan;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;

pub struct ElementScanExecutor;

/// Implementation of a task that performs a scan against the provided snapshot. Does not collect new values.
/// Caller is assumed to have already done this if desired.
impl ElementScanExecutor {
    pub fn execute_scan(
        process_info: OpenedProcessInfo,
        snapshot: Arc<RwLock<Snapshot>>,
        element_scan_plan: ElementScanPlan,
        with_logging: bool,
        scan_execution_context: &ScanExecutionContext,
    ) {
        Self::scan_task(process_info, snapshot, element_scan_plan, with_logging, scan_execution_context);
    }

    fn scan_task(
        process_info: OpenedProcessInfo,
        snapshot: Arc<RwLock<Snapshot>>,
        element_scan_plan: ElementScanPlan,
        with_logging: bool,
        scan_execution_context: &ScanExecutionContext,
    ) {
        let total_start_time = Instant::now();

        // If the parameter is set, first collect values before the scan.
        // This is slower overall than interleaving the reads, but better for capturing values that may soon change.
        if element_scan_plan.get_memory_read_mode() == MemoryReadMode::ReadBeforeScan {
            ValueCollector::collect_values(process_info.clone(), snapshot.clone(), with_logging, scan_execution_context);
        }

        if with_logging {
            log::info!("Performing manual scan...");
        }

        let mut snapshot = match snapshot.write() {
            Ok(guard) => guard,
            Err(error) => {
                if with_logging {
                    log::error!("Failed to acquire write lock on snapshot: {}", error);
                }

                return;
            }
        };

        let start_time = Instant::now();
        let processed_region_count = Arc::new(AtomicUsize::new(0));
        let total_region_count = snapshot.get_region_count();
        let snapshot_regions = snapshot.get_snapshot_regions_mut();

        // Create a function that processes every snapshot region, from which we will grab the existing snapshot filters (previous results) to perform our next scan.
        let snapshot_iterator = |snapshot_region: &mut SnapshotRegion| {
            if scan_execution_context.should_cancel() {
                return;
            }

            // Creates initial results if none exist yet.
            snapshot_region.initialize_scan_results(element_scan_plan.get_data_type_refs_iterator(), element_scan_plan.get_memory_alignment());

            // Attempt to read new (or initial) memory values. Ignore failures as they usually indicate deallocated pages. // JIRA: Remove failures somehow.
            if element_scan_plan.get_memory_read_mode() == MemoryReadMode::ReadInterleavedWithScan {
                let _ = snapshot_region.read_all_memory(&process_info, scan_execution_context);
            }

            /*
            // JIRA: Fixme? Early exit gains?
            if !element_scan_plan.is_valid_for_snapshot_region(snapshot_region) {
                processed_region_count.fetch_add(1, Ordering::SeqCst);
                return;
            }*/

            // Create a function to dispatch our element scan to the best scanner implementation for the current region.
            let element_scan_dispatcher = |snapshot_region_filter_collection| {
                ElementScanDispatcher::dispatch_scan(snapshot_region, snapshot_region_filter_collection, &element_scan_plan)
            };

            // Again, select the parallel or sequential iterator to iterate over each data type in the scan. Generally there is only 1, but multi-type scans are supported.
            let scan_results_collection = snapshot_region.get_scan_results().get_filter_collections();
            let single_thread_scan = element_scan_plan.get_is_single_thread_scan() || scan_results_collection.len() == 1;
            let scan_results = SnapshotRegionScanResults::new(if single_thread_scan {
                scan_results_collection
                    .iter()
                    .map(element_scan_dispatcher)
                    .collect()
            } else {
                scan_results_collection
                    .par_iter()
                    .map(element_scan_dispatcher)
                    .collect()
            });

            snapshot_region.set_scan_results(scan_results);

            let processed = processed_region_count.fetch_add(1, Ordering::SeqCst);

            // To reduce performance impact, only periodically send progress updates.
            if processed % 32 == 0 {
                let progress = (processed as f32 / total_region_count as f32) * 100.0;
                scan_execution_context.report_progress(progress);
            }
        };

        // Select either the parallel or sequential iterator. Single-thread is not advised unless debugging.
        let single_thread_scan = element_scan_plan.get_is_single_thread_scan() || snapshot_regions.len() == 1;
        if single_thread_scan {
            snapshot_regions.iter_mut().for_each(snapshot_iterator);
        } else {
            snapshot_regions.par_iter_mut().for_each(snapshot_iterator);
        };

        snapshot.discard_empty_regions();

        if with_logging {
            let byte_count = snapshot.get_byte_count();
            let duration = start_time.elapsed();
            let total_duration = total_start_time.elapsed();

            log::info!("Results: {} bytes", StorageSizeConversions::value_to_metric_size(byte_count as u128));
            log::info!("Scan complete in: {:?}", duration);
            log::info!("Total scan time: {:?}", total_duration);
        }
    }
}
