use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection;
use crate::results::snapshot_region_scan_results::SnapshotRegionScanResults;
use crate::snapshots::snapshot::Snapshot;
use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use squalr_engine_common::conversions::Conversions;
use squalr_engine_common::structures::processes::process_info::OpenedProcessInfo;
use squalr_engine_common::tasks::trackable_task::TrackableTask;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

pub struct ValueCollector;

/// Implementation of a task that collects new or initial values for the provided snapshot.
impl ValueCollector {
    const NAME: &'static str = "Value Collector";

    pub fn collect_values(
        process_info: OpenedProcessInfo,
        snapshot: Arc<RwLock<Snapshot>>,
        task_identifier: Option<String>,
        with_logging: bool,
    ) -> Arc<TrackableTask<()>> {
        let process_info = Arc::new(process_info);
        let task = TrackableTask::<()>::create(ValueCollector::NAME.to_string(), task_identifier);
        let task_clone = task.clone();
        let process_info_clone = process_info.clone();
        let snapshot = snapshot.clone();

        std::thread::spawn(move || {
            Self::collect_values_task(
                process_info_clone,
                snapshot,
                with_logging,
                task_clone.clone(),
                task_clone.get_cancellation_token(),
            );

            task_clone.complete(());
        });

        task
    }

    fn collect_values_task(
        process_info: Arc<OpenedProcessInfo>,
        snapshot: Arc<RwLock<Snapshot>>,
        with_logging: bool,
        task: Arc<TrackableTask<()>>,
        cancellation_token: Arc<AtomicBool>,
    ) {
        if with_logging {
            log::info!("Reading values from memory (process {})...", process_info.process_id);
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
        let snapshot_regions = snapshot.get_snapshot_regions_mut();

        // Collect values for each snapshot region in parallel.
        snapshot_regions.par_iter_mut().for_each(|snapshot_region| {
            if cancellation_token.load(Ordering::SeqCst) {
                return;
            }

            // Attempt to read new (or initial) memory values. Ignore failed regions, as these are generally just deallocated pages.
            // JIRA: We probably want some way of tombstoning deallocated pages.
            let _ = snapshot_region.read_all_memory(&process_info);

            // Create the scan results for this region.
            let snapshot_region_scan_results = SnapshotRegionScanResults::new(
                // Iterate over all data type / memory alignment pair in parallel.
                snapshot_region
                    .get_scan_results()
                    .get_filter_collections()
                    .par_iter()
                    .map(|current_filter_collection| {
                        // Create a new filter collection. Because we are just collecting values, the filter will encompass the entire region.
                        // JIRA: This is probably where we would apply tombstoning based on the read above -- we would want to have the filters mask
                        // out the regions with a failed read.
                        SnapshotRegionFilterCollection::new(
                            vec![vec![SnapshotRegionFilter::new(
                                snapshot_region.get_base_address(),
                                snapshot_region.get_region_size(),
                            )]],
                            current_filter_collection.get_scan_parameters_local().clone(),
                        )
                    })
                    .collect(),
            );

            snapshot_region.set_scan_results(snapshot_region_scan_results);

            // Report progress periodically (not every time for performance)
            let processed = processed_region_count.fetch_add(1, Ordering::SeqCst);
            if processed % 32 == 0 {
                let progress = (processed as f32 / total_region_count as f32) * 100.0;
                task.set_progress(progress);
            }
        });

        if with_logging {
            let duration = start_time.elapsed();
            let byte_count = snapshot.get_byte_count();

            log::info!("Values collected in: {:?}", duration);
            log::info!("{} bytes read ({})", byte_count, Conversions::value_to_metric_size(byte_count));
        }
    }
}
