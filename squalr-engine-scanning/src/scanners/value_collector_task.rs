use crate::snapshots::snapshot::Snapshot;
use crate::snapshots::snapshot_region::SnapshotRegion;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::iter::ParallelIterator;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::tasks::trackable_task::TrackableTask;
use squalr_engine_common::conversions::Conversions;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

const TASK_NAME: &'static str = "Value Collector";

pub struct ValueCollectorTask;

/// Implementation of a task that collects new or initial values for the provided snapshot.
impl ValueCollectorTask {
    pub fn start_task(
        process_info: OpenedProcessInfo,
        snapshot: Arc<RwLock<Snapshot>>,
        with_logging: bool,
    ) -> Arc<TrackableTask> {
        let task = TrackableTask::create(TASK_NAME.to_string(), None);
        let task_clone = task.clone();
        let process_info = Arc::new(process_info);
        let process_info_clone = process_info.clone();
        let snapshot = snapshot.clone();

        std::thread::spawn(move || {
            Self::collect_values_task(&task_clone, process_info_clone, snapshot, with_logging);

            task_clone.complete();
        });

        task
    }

    fn collect_values_task(
        trackable_task: &Arc<TrackableTask>,
        process_info: Arc<OpenedProcessInfo>,
        snapshot: Arc<RwLock<Snapshot>>,
        with_logging: bool,
    ) {
        if with_logging {
            log::info!("Reading values from memory (process {})...", process_info.get_process_id_raw());
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

        let snapshot_regions = snapshot.get_snapshot_regions_mut();
        let cancellation_token = trackable_task.get_cancellation_token();

        let read_memory_iterator = |snapshot_region: &mut SnapshotRegion| {
            if cancellation_token.load(Ordering::SeqCst) {
                return;
            }

            // Attempt to read new (or initial) memory values. Ignore failed regions, as these are generally just deallocated pages.
            // JIRA: We probably want some way of tombstoning deallocated pages.
            let _ = snapshot_region.read_all_memory(&process_info);

            // Report progress periodically (not every time for performance)
            let processed = processed_region_count.fetch_add(1, Ordering::SeqCst);

            if processed % 32 == 0 {
                let progress = (processed as f32 / total_region_count as f32) * 100.0;
                trackable_task.set_progress(progress);
            }
        };

        // Collect values for each snapshot region in parallel.
        snapshot_regions.par_iter_mut().for_each(read_memory_iterator);

        if with_logging {
            let duration = start_time.elapsed();
            let byte_count = snapshot.get_byte_count();

            log::info!("Values collected in: {:?}", duration);
            log::info!("{} bytes read ({})", byte_count, Conversions::value_to_metric_size(byte_count));
        }
    }
}
