use crate::scanners::scan_execution_context::ScanExecutionContext;
use crate::scanners::value_collector_task::ValueCollector;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::scanning::plans::pointer_scan::pointer_scan_parameters::PointerScanParameters;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use std::sync::{Arc, RwLock};

pub struct PointerScanExecutor;

/// Implementation of a task that performs a scan against the provided snapshot. Does not collect new values.
/// Caller is assumed to have already done this if desired.
impl PointerScanExecutor {
    pub fn execute_scan(
        process_info: OpenedProcessInfo,
        statics_snapshot: Arc<RwLock<Snapshot>>,
        heaps_snapshot: Arc<RwLock<Snapshot>>,
        pointer_scan_parameters: PointerScanParameters,
        with_logging: bool,
        scan_execution_context: &ScanExecutionContext,
    ) {
        Self::scan_task(
            process_info,
            statics_snapshot,
            heaps_snapshot,
            pointer_scan_parameters,
            with_logging,
            scan_execution_context,
        );
    }

    fn scan_task(
        process_info: OpenedProcessInfo,
        statics_snapshot: Arc<RwLock<Snapshot>>,
        heaps_snapshot: Arc<RwLock<Snapshot>>,
        _pointer_scan_parameters: PointerScanParameters,
        with_logging: bool,
        scan_execution_context: &ScanExecutionContext,
    ) {
        if with_logging {
            log::info!("Performing pointer scan...");
        }

        // Populate the latest static and heap values from process memory.
        ValueCollector::collect_values(process_info.clone(), statics_snapshot.clone(), with_logging, scan_execution_context);
        ValueCollector::collect_values(process_info.clone(), heaps_snapshot.clone(), with_logging, scan_execution_context);

        // Find valid pointers. JIRA: Binary search kernel?
        /*
        let pointer_scan_minimum_address = ElementScanValue::new(DataTypeU64::get_value_from_primitive(0), MemoryAlignment::Alignment4);
        let element_scan_parameters = ElementScanParameters::new(
            Vec::new(),
            ScanCompareType::Immediate(ScanCompareTypeImmediate::GreaterThan),
            vec![pointer_scan_minimum_address],
            FloatingPointTolerance::default(),
            MemoryReadMode::Skip,
            pointer_scan_parameters.get_is_single_thread_scan(),
            pointer_scan_parameters.get_debug_perform_validation_scan(),
        );
        ElementScanExecutorTask::start_task(
            process_info.clone(),
            statics_snapshot.clone(),
            element_scan_rule_registry.clone(),
            symbol_registry.clone(),
            element_scan_parameters.clone(),
            with_logging,
        )
        .wait_for_completion();
        ElementScanExecutorTask::start_task(
            process_info.clone(),
            heaps_snapshot.clone(),
            element_scan_rule_registry,
            symbol_registry,
            element_scan_parameters,
            with_logging,
        )
        .wait_for_completion();

        let mut statics_snapshot = match statics_snapshot.write() {
            Ok(guard) => guard,
            Err(error) => {
                if with_logging {
                    log::error!("Failed to acquire write lock on statics_snapshot: {}", error);
                }

                return;
            }
        };
        /*
        let mut heaps_snapshot = match heaps_snapshot.write() {
            Ok(guard) => guard,
            Err(error) => {
                if with_logging {
                    log::error!("Failed to acquire write lock on heaps_snapshot: {}", error);
                }

                return;
            }
        };
        */

        let start_time = Instant::now();
        let processed_region_count = Arc::new(AtomicUsize::new(0));
        let total_region_count = statics_snapshot.get_region_count();
        let cancellation_token = trackable_task.get_cancellation_token();
        let snapshot_regions = statics_snapshot.get_snapshot_regions_mut();

        // Create a function that processes every snapshot region, from which we will grab the existing snapshot filters (previous results) to perform our next scan.
        let snapshot_iterator = |snapshot_region: &mut SnapshotRegion| {
            if cancellation_token.load(Ordering::SeqCst) {
                return;
            }

            /*
            // Create a function to dispatch our element scan to the best scanner implementation for the current region.
            let pointer_scan_dispatcher = |snapshot_region_filter_collection| {
                PointerScanDispatcher::dispatch_scan(
                    &pointer_scan_rule_registry,
                    &symbol_registry,
                    snapshot_region,
                    snapshot_region_filter_collection,
                    &pointer_scan_parameters,
                )
            };

            // Again, select the parallel or sequential iterator to iterate over each data type in the scan. Generally there is only 1, but multi-type scans are supported.
            let scan_results_collection = snapshot_region.get_scan_results().get_filter_collections();
            let single_thread_scan = pointer_scan_parameters.get_is_single_thread_scan() || scan_results_collection.len() == 1;
            let scan_results = SnapshotRegionScanResults::new(if single_thread_scan {
                scan_results_collection
                    .iter()
                    .map(pointer_scan_dispatcher)
                    .collect()
            } else {
                scan_results_collection
                    .par_iter()
                    .map(pointer_scan_dispatcher)
                    .collect()
            });

            snapshot_region.set_scan_results(scan_results);*/

            let processed = processed_region_count.fetch_add(1, Ordering::SeqCst);

            // To reduce performance impact, only periodically send progress updates.
            if processed % 32 == 0 {
                let progress = (processed as f32 / total_region_count as f32) * 100.0;
                trackable_task.set_progress(progress);
            }
        };

        // Select either the parallel or sequential iterator. Single-thread is not advised unless debugging.
        let single_thread_scan = pointer_scan_parameters.get_is_single_thread_scan() || snapshot_regions.len() == 1;
        if single_thread_scan {
            snapshot_regions.iter_mut().for_each(snapshot_iterator);
        } else {
            snapshot_regions.par_iter_mut().for_each(snapshot_iterator);
        };

        statics_snapshot.discard_empty_regions();

        if with_logging {
            let byte_count = statics_snapshot.get_byte_count();
            let duration = start_time.elapsed();
            let total_duration = total_start_time.elapsed();

            log::info!("Results: {} bytes", Conversions::value_to_metric_size(byte_count));
            log::info!("Scan complete in: {:?}", duration);
            log::info!("Total scan time: {:?}", total_duration);
        }*/
    }
}
