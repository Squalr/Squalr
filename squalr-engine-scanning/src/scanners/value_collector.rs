use crate::snapshots::snapshot::Snapshot;
use crate::snapshots::snapshot_region::SnapshotRegion;

use futures::future::join_all;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_processes::process_info::ProcessInfo;
use squalr_engine_common::tasks::trackable_task::TrackableTask;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Instant;
use tokio::task::JoinHandle;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct ValueCollector;

impl ValueCollector {
    const NAME: &'static str = "Value Collector";

    pub fn collect_values(
        process_info: ProcessInfo,
        snapshot: Arc<RwLock<Snapshot>>,
        task_identifier: Option<String>,
        with_logging: bool,
    ) -> Arc<TrackableTask<()>> {
        let task = TrackableTask::<()>::create(
            ValueCollector::NAME.to_string(),
            Some(Uuid::new_v4()),
        );

        let process_info = Arc::new(process_info);

        let task_handle: JoinHandle<()> = tokio::spawn({
            let task = task.clone();
            let process_info = process_info.clone();
            let snapshot = snapshot.clone();
            async move {
                Self::collect_values_task(
                    process_info,
                    snapshot,
                    with_logging,
                    task.progress_sender.clone(),
                    task.cancellation_token(),
                ).await;
                
                task.complete(());
            }
        });

        task.add_handle(task_handle);

        return task;
    }

    async fn collect_values_task(
        process_info: Arc<ProcessInfo>,
        snapshot: Arc<RwLock<Snapshot>>,
        with_logging: bool,
        progress_sender: broadcast::Sender<f32>,
        cancellation_token: CancellationToken,
    ) {
        let region_count;
        let snapshot_regions;

        // Lock the snapshot briefly to extract the regions
        {
            let mut snapshot = snapshot.write().unwrap();
            region_count = snapshot.snapshot_regions.len();
            snapshot_regions = std::mem::take(&mut snapshot.snapshot_regions);
        }

        if with_logging {
            Logger::instance().log(LogLevel::Info, "Reading values from memory...", None);
        }

        let start_time = Instant::now();
        let processed_region_count = Arc::new(AtomicUsize::new(0));

        let results: Vec<JoinHandle<Option<SnapshotRegion>>> = snapshot_regions.into_iter().map(|mut region| {
            let processed_region_count = processed_region_count.clone();
            let progress_sender = progress_sender.clone();
            let cancellation_token = cancellation_token.clone();
            let process_info = process_info.clone();

            tokio::spawn(async move {
                if cancellation_token.is_cancelled() {
                    return None;
                }

                region.read_all_memory(process_info.handle).unwrap();

                let processed = processed_region_count.fetch_add(1, Ordering::SeqCst);
                if processed % 32 == 0 {
                    let progress = (processed as f32 / region_count as f32) * 100.0;
                    let _ = progress_sender.send(progress);
                }

                Some(region)
            })
        }).collect();
        
        let results = join_all(results).await.into_iter().filter_map(|x| x.unwrap()).collect::<Vec<SnapshotRegion>>();
        let byte_count: u64 = results.iter().map(|r| r.get_region_size()).sum();

        // Lock the snapshot briefly to update it
        {
            let mut snapshot = snapshot.write().unwrap();
            snapshot.set_snapshot_regions(results);
        }

        let duration = start_time.elapsed();

        if with_logging {
            Logger::instance().log(LogLevel::Info, &format!("Values collected in: {:?}", duration), None);
            Logger::instance().log(LogLevel::Info, &format!("{} bytes read", byte_count), None);
        }
    }
}
