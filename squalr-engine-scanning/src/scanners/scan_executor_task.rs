use crate::results::snapshot_region_scan_results::SnapshotRegionScanResults;
use crate::scanners::scan_dispatcher::ScanDispatcher;
use crate::scanners::value_collector_task::ValueCollectorTask;
use crate::snapshots::snapshot::Snapshot;
use crate::snapshots::snapshot_region::SnapshotRegion;
use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use squalr_engine_api::structures::processes::process_info::OpenedProcessInfo;
use squalr_engine_api::structures::scanning::memory_read_mode::MemoryReadMode;
use squalr_engine_api::structures::scanning::parameters::user::user_scan_parameters_global::UserScanParametersGlobal;
use squalr_engine_api::structures::tasks::trackable_task::TrackableTask;
use squalr_engine_common::conversions::Conversions;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Instant;

pub struct ScanExecutorTask;

const TASK_NAME: &'static str = "Scan Executor";

/// Implementation of a task that performs a scan against the provided snapshot. Does not collect new values.
/// Caller is assumed to have already done this if desired.
impl ScanExecutorTask {
    pub fn start_task(
        process_info: OpenedProcessInfo,
        snapshot: Arc<RwLock<Snapshot>>,
        user_scan_parameters_global: &UserScanParametersGlobal,
        with_logging: bool,
    ) -> Arc<TrackableTask> {
        let task = TrackableTask::create(TASK_NAME.to_string(), None);
        let task_clone = task.clone();
        let user_scan_parameters_global_clone = user_scan_parameters_global.clone();

        thread::spawn(move || {
            Self::scan_task(&task_clone, process_info, snapshot, &user_scan_parameters_global_clone, with_logging);

            task_clone.complete();
        });

        task
    }

    fn scan_task(
        trackable_task: &Arc<TrackableTask>,
        process_info: OpenedProcessInfo,
        snapshot: Arc<RwLock<Snapshot>>,
        user_scan_parameters_global: &UserScanParametersGlobal,
        with_logging: bool,
    ) {
        let total_start_time = Instant::now();

        // If the parameter is set, first collect values before the scan.
        // This is slower overall than interleaving the reads, but better for capturing values that may soon change.
        if user_scan_parameters_global.get_memory_read_mode() == MemoryReadMode::ReadBeforeScan {
            ValueCollectorTask::start_task(process_info.clone(), snapshot.clone(), with_logging).wait_for_completion();
        }

        if with_logging {
            log::info!("Performing manual scan...");
        }

        let mut snapshot = match snapshot.write() {
            Ok(guard) => guard,
            Err(err) => {
                if with_logging {
                    log::error!("Failed to acquire write lock on snapshot: {}", err);
                }

                return;
            }
        };

        let start_time = Instant::now();
        let processed_region_count = Arc::new(AtomicUsize::new(0));
        let total_region_count = snapshot.get_region_count();
        let cancellation_token = trackable_task.get_cancellation_token();
        let snapshot_regions = snapshot.get_snapshot_regions_mut();

        // Create a function that processes every snapshot region, from which we will grab the existing snapshot filters (previous results) to perform our next scan.
        let snapshot_iterator = |snapshot_region: &mut SnapshotRegion| {
            if cancellation_token.load(Ordering::SeqCst) {
                return;
            }

            // Attempt to read new (or initial) memory values. Ignore failures as they usually indicate deallocated pages. // JIRA: Remove failures somehow.
            if user_scan_parameters_global.get_memory_read_mode() == MemoryReadMode::ReadInterleavedWithScan {
                let _ = snapshot_region.read_all_memory(&process_info);
            }

            // Skip the region if it is impossible to perform the scan (ie previous values are missing).
            if !snapshot_region.can_compare_using_parameters(user_scan_parameters_global) {
                processed_region_count.fetch_add(1, Ordering::SeqCst);
                return;
            }

            // Create a function to dispatch our scan to the best scanner implementation for the current region.
            let scan_dispatcher = |snapshot_region_filter_collection| {
                ScanDispatcher::dispatch_scan(snapshot_region, snapshot_region_filter_collection, user_scan_parameters_global)
            };

            // Again, select the parallel or sequential iterator to iterate over each data type in the scan. Generally there is only 1, but multi-type scans are supported.
            let scan_results_collection = snapshot_region.get_scan_results().get_filter_collections();
            let single_thread_scan = user_scan_parameters_global.is_single_thread_scan() || scan_results_collection.len() == 1;
            let scan_results = SnapshotRegionScanResults::new(if single_thread_scan {
                scan_results_collection.iter().map(scan_dispatcher).collect()
            } else {
                scan_results_collection
                    .par_iter()
                    .map(scan_dispatcher)
                    .collect()
            });

            snapshot_region.set_scan_results(scan_results);

            let processed = processed_region_count.fetch_add(1, Ordering::SeqCst);

            // To reduce performance impact, only periodically send progress updates.
            if processed % 32 == 0 {
                let progress = (processed as f32 / total_region_count as f32) * 100.0;
                trackable_task.set_progress(progress);
            }
        };

        // Select either the parallel or sequential iterator. Single-thread is not advised unless debugging.
        let single_thread_scan = user_scan_parameters_global.is_single_thread_scan() || snapshot_regions.len() == 1;
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

            log::info!("Results: {} bytes", Conversions::value_to_metric_size(byte_count));
            log::info!("Scan complete in: {:?}", duration);
            log::info!("Total scan time: {:?}", total_duration);
        }
    }
}
