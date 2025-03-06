use crate::results::snapshot_region_scan_results::SnapshotRegionScanResults;
use crate::scanners::scan_dispatcher::ScanDispatcher;
use crate::snapshots::snapshot::Snapshot;
use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use squalr_engine_common::conversions::Conversions;
use squalr_engine_common::structures::scanning::scan_parameters::ScanParameters;
use squalr_engine_common::tasks::trackable_task::TrackableTask;
use std::sync::atomic::{AtomicUsize, Ordering};
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
            let scan_results = Self::scan_task(snapshot, &scan_parameters_clone, task_clone.clone(), with_logging);

            task_clone.complete(scan_results);
        });

        task
    }

    fn scan_task(
        snapshot: Arc<RwLock<Snapshot>>,
        scan_parameters: &ScanParameters,
        task: Arc<TrackableTask<()>>,
        with_logging: bool,
    ) {
        if with_logging {
            log::info!("Performing manual scan...");
        }

        let mut snapshot = match snapshot.write() {
            Ok(guard) => guard,
            Err(e) => {
                if with_logging {
                    log::error!("Failed to acquire write lock on snapshot: {}", e);
                }

                return;
            }
        };

        let start_time = Instant::now();
        let processed_region_count = Arc::new(AtomicUsize::new(0));
        let total_region_count = snapshot.get_region_count();
        let cancellation_token = task.get_cancellation_token();

        // Iterate over every snapshot region, from which we will grab the existing snapshot filters to perform our next scan.
        snapshot
            .get_snapshot_regions_mut()
            .par_iter_mut()
            .for_each(|snapshot_region| {
                if cancellation_token.load(Ordering::SeqCst) {
                    return;
                }

                if !snapshot_region.can_compare_using_parameters(scan_parameters) {
                    processed_region_count.fetch_add(1, Ordering::SeqCst);
                    return;
                }

                // Iterate over each data type in the scan. Generally there is only 1, but multiple simultaneous scans are supported.
                let scan_results = SnapshotRegionScanResults::new(
                    snapshot_region
                        .get_scan_results()
                        .get_filter_collections()
                        .par_iter()
                        .map(|snapshot_region_filter_collection| {
                            // Perform the scan.
                            ScanDispatcher::dispatch_scan_parallel(snapshot_region, snapshot_region_filter_collection, scan_parameters)
                        })
                        .collect(),
                );

                snapshot_region.set_scan_results(scan_results);

                let processed = processed_region_count.fetch_add(1, Ordering::SeqCst);

                // To reduce performance impact, only periodically send progress updates.
                if processed % 32 == 0 {
                    let progress = (processed as f32 / total_region_count as f32) * 100.0;
                    task.set_progress(progress);
                }
            });

        snapshot.discard_empty_regions();

        if with_logging {
            let byte_count = snapshot.get_byte_count();
            let duration = start_time.elapsed();

            log::info!("Results: {} bytes", Conversions::value_to_metric_size(byte_count));

            /*
            let scan_results = snapshot.get_scan_results_by_data_type();

            for (data_type, _) in data_types_and_alignments {
                if let Some(scan_results_for_type) = scan_results.get(&data_type) {
                    let element_count = scan_results_for_type.get_number_of_results();
                    log::info!("Results [{:?}]: {} element(s)", data_type, element_count);
                }
            }*/

            log::info!("Scan complete in: {:?}", duration);
        }
    }
}
