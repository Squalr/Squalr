use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::scanners::scan_dispatcher::ScanDispatcher;
use crate::snapshots::snapshot::Snapshot;
use rayon::iter::{IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use squalr_engine_common::conversions::value_to_metric_size;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::tasks::trackable_task::TrackableTask;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Instant;

pub struct ManualScanner;

/// Implementation of a task that performs a scan against the provided snapshot. Does not collect new values.
/// Caller is assumed to have already done this if desired.
impl ManualScanner {
    const NAME: &'static str = "Manual Scan";

    pub fn scan(
        snapshot: Arc<RwLock<Snapshot>>,
        scan_parameters: &ScanParameters,
        task_identifier: Option<String>,
        with_logging: bool,
    ) -> Arc<TrackableTask<()>> {
        let task = TrackableTask::<()>::create(ManualScanner::NAME.to_string(), task_identifier);

        let task_clone = task.clone();
        let scan_parameters_clone = scan_parameters.clone();

        thread::spawn(move || {
            Self::scan_task(
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
        snapshot: Arc<RwLock<Snapshot>>,
        scan_parameters: &ScanParameters,
        task: Arc<TrackableTask<()>>,
        cancellation_token: Arc<AtomicBool>,
        with_logging: bool,
    ) {
        if with_logging {
            Logger::get_instance().log(LogLevel::Info, "Performing manual scan...", None);
        }

        let region_count = snapshot.read().unwrap().get_region_count();
        let scan_parameters_filters = snapshot.read().unwrap().get_scan_parameters_filters().clone();
        let mut snapshot = snapshot.write().unwrap();
        let snapshot_regions = snapshot.get_snapshot_regions_for_update();
        let scan_parameters = &scan_parameters.clone();
        let start_time = Instant::now();
        let processed_region_count = Arc::new(AtomicUsize::new(0));

        // Iterate over every snapshot region, from which we will grab the existing snapshot filters to perform our next scan.
        snapshot_regions.par_iter_mut().for_each(|snapshot_region| {
            if cancellation_token.load(Ordering::SeqCst) {
                return;
            }

            if !snapshot_region.can_compare_using_parameters(scan_parameters) {
                processed_region_count.fetch_add(1, Ordering::SeqCst);
                return;
            }

            // Iterate over each data type in the scan. Generally there is only 1, but multiple simultaneous scans are supported.
            let new_filters = scan_parameters_filters
                .clone()
                .into_par_iter()
                .filter_map(|scan_filter_parameter| {
                    let data_type = scan_filter_parameter.get_data_type();

                    let snapshot_region_filters_map = snapshot_region.get_filters();
                    let snapshot_region_filters = snapshot_region_filters_map.get(data_type);

                    if snapshot_region_filters.is_none() {
                        return None;
                    }

                    let snapshot_region_filters = snapshot_region_filters.unwrap();
                    let scan_dispatcher = ScanDispatcher::get_instance();
                    let scan_results;

                    if snapshot_region_filters.len() > 0 {
                        scan_results =
                            scan_dispatcher.dispatch_scan_parallel(snapshot_region, snapshot_region_filters, scan_parameters, &scan_filter_parameter);
                    } else {
                        scan_results = scan_dispatcher.dispatch_scan(snapshot_region, snapshot_region_filters, scan_parameters, &scan_filter_parameter);
                    }

                    return Some((data_type.clone(), scan_results));
                })
                .collect();

            // Update the snapshot region to contain new filtered regions (ie scan results).
            snapshot_region.set_all_filters(new_filters);

            let processed = processed_region_count.fetch_add(1, Ordering::SeqCst);

            // To reduce performance impact, only periodically send progress updates.
            if processed % 32 == 0 {
                let progress = (processed as f32 / region_count as f32) * 100.0;
                task.set_progress(progress);
            }
        });

        snapshot.discard_empty_regions();

        let byte_count = snapshot.get_byte_count();
        let duration = start_time.elapsed();

        if with_logging {
            Logger::get_instance().log(LogLevel::Info, &format!("Results: {} bytes", value_to_metric_size(byte_count)), None);
            Logger::get_instance().log(LogLevel::Info, &format!("Scan complete in: {:?}", duration), None);
        }
    }
}
