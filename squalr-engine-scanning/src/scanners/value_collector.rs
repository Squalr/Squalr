use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use sysinfo::Pid;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use futures::future::join_all;

use crate::snapshots::snapshot::Snapshot;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::tasks::trackable_task::TrackableTask;

type UpdateProgress = Arc<dyn Fn(f32) + Send + Sync>;

pub struct ValueCollector;

impl ValueCollector {
    const NAME: &'static str = "Value Collector";

    pub fn collect_values(
        process_id: Pid,
        snapshot: Arc<Mutex<Snapshot>>,
        task_identifier: Option<String>,
        // optional_constraint: Option<ScanConstraints>,
        with_logging: bool,
    ) -> Arc<TrackableTask<Snapshot>> {
        let task = TrackableTask::<Snapshot>::create(
            ValueCollector::NAME.to_string(),
            Some(Uuid::new_v4()),
            with_logging,
        );

        let snapshot_clone = snapshot.clone();

        let task_handle: JoinHandle<()> = tokio::spawn({
            let task = task.clone();
            async move {
                let result = Self::collect_values_task(
                    process_id,
                    snapshot_clone,
                    // optional_constraint,
                    with_logging,
                    task.progress_callback(),
                    task.cancellation_token(),
                )
                .await;

                task.complete(result);
            }
        });

        task.add_handle(task_handle);

        return task;
    }

    async fn collect_values_task(
        process_id: Pid,
        snapshot: Arc<Mutex<Snapshot>>,
        // optional_constraint: Option<ScanConstraints>,
        with_logging: bool,
        update_progress: UpdateProgress,
        cancellation_token: CancellationToken,
    ) -> Snapshot {
        let processed_regions = Arc::new(AtomicUsize::new(0));
        let snapshot = snapshot.lock().unwrap().clone();
        let total_regions = snapshot.get_region_count();

        if with_logging {
            Logger::instance().log(LogLevel::Info, "Reading values from memory...", None);
        }

        let start_time = Instant::now();

        let regions: Vec<&SnapshotRegion> = snapshot.get_optimal_sorted_snapshot_regions().collect();
        let results: Vec<SnapshotRegion> = join_all(
            regions.into_iter().map(|region| {
                let process_id = process_id.clone();
                // let optional_constraint = optional_constraint.clone();
                let update_progress = update_progress.clone();
                let cancellation_token = cancellation_token.clone();
                let processed_regions = processed_regions.clone();
                async move {
                    if cancellation_token.is_cancelled() {
                        return None;
                    }

                    let mut region = region.clone();
                    region.read_all_memory(&process_id).unwrap();

                    // if let Some(constraint) = &optional_constraint {
                    //     region.set_alignment(constraint.alignment, constraint.element_size);
                    // }

                    let processed = processed_regions.fetch_add(1, Ordering::SeqCst);
                    if processed % 32 == 0 {
                        let progress = (processed as f32 / total_regions as f32) * 100.0;
                        update_progress(progress);
                    }

                    Some(region)
                }
            })
        )
        .await
        .into_iter()
        .filter_map(|x| x)
        .collect();

        let byte_count: u64 = results.iter().map(|r| r.get_region_size()).sum();

        let mut new_snapshot = snapshot.clone();
        new_snapshot.set_snapshot_regions(results);

        // if optional_constraint.is_some() {
            // new_snapshot.set_alignment(optional_constraint.unwrap().alignment);
        // }

        let duration = start_time.elapsed();

        if with_logging {
            Logger::instance().log(LogLevel::Info, &format!("Values collected in: {:?}" , duration), None);
            Logger::instance().log(LogLevel::Info,  &format!("{} bytes read", byte_count), None);
        }

        new_snapshot
    }
}
