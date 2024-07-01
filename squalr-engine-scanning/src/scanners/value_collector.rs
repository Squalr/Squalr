use crate::snapshots::snapshot::Snapshot;
use crate::snapshots::snapshot_region::SnapshotRegion;

use futures::future::join_all;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_processes::process_info::ProcessInfo;
use squalr_engine_common::tasks::trackable_task::TrackableTask;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

type UpdateProgress = Arc<dyn Fn(f32) + Send + Sync>;

pub struct ValueCollector;

impl ValueCollector {
    const NAME: &'static str = "Value Collector";

    pub fn collect_values(
        process_info: ProcessInfo,
        snapshot: Box<Snapshot>,
        task_identifier: Option<String>,
        // optional_constraint: Option<ScanConstraints>,
        with_logging: bool,
    ) -> Arc<TrackableTask<()>> {
        let task = TrackableTask::<()>::create(
            ValueCollector::NAME.to_string(),
            Some(Uuid::new_v4()),
            with_logging,
        );

        let process_info = Arc::new(process_info);
        let snapshot = Arc::new(Mutex::new(snapshot));

        let task_handle: JoinHandle<()> = tokio::spawn({
            let task = task.clone();
            
            async move {
                let result = Self::collect_values_task(
                    process_info,
                    snapshot,
                    // optional_constraint,
                    with_logging,
                    task.progress_callback(),
                    task.cancellation_token(),
                ).await;
                
                task.complete(result);
            }
        });

        task.add_handle(task_handle);

        return task;
    }

    async fn collect_values_task(
        process_info: Arc<ProcessInfo>,
        snapshot: Arc<Mutex<Box<Snapshot>>>,
        // optional_constraint: Option<ScanConstraints>,
        with_logging: bool,
        update_progress: UpdateProgress,
        cancellation_token: CancellationToken,
    ) {
        let processed_regions = Arc::new(AtomicUsize::new(0));
        let region_count;
        let snapshot_regions;

        // Lock the snapshot briefly to extract the regions
        {
            let mut snapshot = snapshot.lock().unwrap();
            region_count = snapshot.snapshot_regions.len();
            snapshot_regions = std::mem::take(&mut snapshot.snapshot_regions);
        }

        if with_logging {
            Logger::instance().log(LogLevel::Info, "Reading values from memory...", None);
        }

        let start_time = Instant::now();

        let results: Vec<JoinHandle<Option<SnapshotRegion>>> = snapshot_regions.into_iter().map(|mut region| {
            // let optional_constraint = optional_constraint.clone();
            let update_progress = update_progress.clone();
            let cancellation_token = cancellation_token.clone();
            let processed_regions = processed_regions.clone();
            let process_info = process_info.clone();

            tokio::spawn(async move {
                if cancellation_token.is_cancelled() {
                    return None;
                }

                region.read_all_memory(process_info.handle).unwrap();

                // if let Some(constraint) = &optional_constraint {
                //     region.set_alignment(constraint.alignment, constraint.element_size);
                // }

                let processed = processed_regions.fetch_add(1, Ordering::SeqCst);
                if processed % 32 == 0 {
                    let progress = (processed as f32 / region_count as f32) * 100.0;
                    update_progress(progress);
                }

                Some(region)
            })
        }).collect();

        let results = join_all(results).await.into_iter().filter_map(|x| x.unwrap()).collect::<Vec<SnapshotRegion>>();

        let byte_count: u64 = results.iter().map(|r| r.get_region_size()).sum();

        // Lock the snapshot briefly to update it
        {
            let mut snapshot = snapshot.lock().unwrap();
            snapshot.set_snapshot_regions(results);

            // if optional_constraint.is_some() {
            //     snapshot.set_alignment(optional_constraint.unwrap().alignment);
            // }
        }

        let duration = start_time.elapsed();

        if with_logging {
            Logger::instance().log(LogLevel::Info, &format!("Values collected in: {:?}", duration), None);
            Logger::instance().log(LogLevel::Info, &format!("{} bytes read", byte_count), None);
        }
    }
}
