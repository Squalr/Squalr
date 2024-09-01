use crate::results::snapshot_region_scan_results::SnapshotRegionScanResults;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::scanners::scan_dispatcher::ScanDispatcher;
use crate::snapshots::snapshot::Snapshot;
use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use squalr_engine_common::conversions::value_to_metric_size;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::tasks::trackable_task::TrackableTask;
use squalr_engine_processes::process_info::ProcessInfo;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Instant;

pub struct HybridScanner;

/// Implementation of a task that collects values and performs a scan in the same thread pool. This is much faster than doing value
/// collection and scanning separately (as ManualScanner does), however this means that regions processed last will not have their
/// values collected until potentially much later than the scan was initiated.
impl HybridScanner {
    const NAME: &'static str = "Hybrid Scan";

    pub fn scan(
        process_info: ProcessInfo,
        snapshot: Arc<RwLock<Snapshot>>,
        scan_parameters: &ScanParameters,
        task_identifier: Option<String>,
        with_logging: bool,
    ) -> Arc<TrackableTask<()>> {
        let task = TrackableTask::<()>::create(HybridScanner::NAME.to_string(), task_identifier);

        let task_clone = task.clone();
        let scan_parameters_clone = scan_parameters.clone();

        thread::spawn(move || {
            Self::scan_task(
                process_info,
                snapshot,
                &scan_parameters_clone,
                task_clone.clone(),
                task_clone.get_cancellation_token().clone(),
                with_logging,
            );

            task_clone.complete(());
        });

        return task;
    }

    fn scan_task(
        process_info: ProcessInfo,
        snapshot: Arc<RwLock<Snapshot>>,
        scan_parameters: &ScanParameters,
        task: Arc<TrackableTask<()>>,
        cancellation_token: Arc<AtomicBool>,
        with_logging: bool,
    ) {
        if with_logging {
            Logger::get_instance().log(LogLevel::Info, "Performing hybrid manual scan...", None);
        }

        let data_types = {
            let snapshot = snapshot.read().unwrap();
            snapshot.get_data_types()
        };
        let data_types_and_alignments = {
            let snapshot = snapshot.read().unwrap();
            snapshot.get_data_types_and_alignments()
        };
        let region_count = snapshot.read().unwrap().get_region_count();
        let mut snapshot = snapshot.write().unwrap();
        let scan_parameters = &scan_parameters.clone();
        let snapshot_regions = snapshot.get_snapshot_regions_for_update();

        let start_time = Instant::now();
        let processed_region_count = Arc::new(AtomicUsize::new(0));

        snapshot_regions.par_iter_mut().for_each(|snapshot_region| {
            if cancellation_token.load(Ordering::SeqCst) {
                return;
            }

            // Attempt to read new (or initial) memory values. Ignore failures as they usually indicate deallocated pages.
            let _ = snapshot_region.read_all_memory(process_info.handle);

            if !snapshot_region.can_compare_using_parameters(scan_parameters) {
                processed_region_count.fetch_add(1, Ordering::SeqCst);
                return;
            }

            snapshot_region.create_initial_scan_results(&data_types);

            // Iterate over each data type in the scan. Generally there is only 1, but multiple simultaneous scans are supported.
            let new_snapshot_region_filters = data_types_and_alignments
                .par_iter()
                .filter_map(|(data_type, memory_alignment)| {
                    // Extract the filters from the previous scan to use in this scan.
                    let previous_scan_results = snapshot_region.get_scan_results();
                    let snapshot_region_filters_map = previous_scan_results.get_filters();
                    let snapshot_region_filters = snapshot_region_filters_map.get(&data_type);

                    if snapshot_region_filters.is_none() {
                        return None;
                    }

                    let snapshot_region_filters = snapshot_region_filters.unwrap();
                    let scan_dispatcher = ScanDispatcher::get_instance();
                    let scan_results;

                    if snapshot_region_filters.len() > 0 {
                        scan_results =
                            scan_dispatcher.dispatch_scan_parallel(snapshot_region, snapshot_region_filters, scan_parameters, &data_type, *memory_alignment);
                    } else {
                        scan_results = scan_dispatcher.dispatch_scan(snapshot_region, snapshot_region_filters, scan_parameters, &data_type, *memory_alignment);
                    }

                    return Some((data_type.clone(), scan_results));
                })
                .collect();

            snapshot_region.set_scan_results(SnapshotRegionScanResults::new_from_filters(new_snapshot_region_filters));

            let processed = processed_region_count.fetch_add(1, Ordering::SeqCst);

            // To reduce performance impact, only periodically send progress updates.
            if processed % 32 == 0 {
                let progress = (processed as f32 / region_count as f32) * 100.0;
                task.set_progress(progress);
            }
        });

        snapshot.discard_empty_regions();
        snapshot.build_scan_results();

        if with_logging {
            let byte_count = snapshot.get_byte_count();
            let duration = start_time.elapsed();

            Logger::get_instance().log(LogLevel::Info, &format!("Results: {} bytes", value_to_metric_size(byte_count)), None);

            let scan_results = snapshot.get_scan_results_by_data_type();

            for data_type in data_types {
                if let Some(scan_results_for_type) = scan_results.get(&data_type) {
                    let element_count = scan_results_for_type.get_number_of_results();
                    Logger::get_instance().log(LogLevel::Info, &format!("Results [{:?}]: {} element(s)", data_type, element_count), None);
                }
            }

            Logger::get_instance().log(LogLevel::Info, &format!("Scan complete in: {:?}", duration), None);
        }
    }
}
