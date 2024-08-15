use crate::scanners::comparers::scan_dispatcher::ScanDispatcher;
use crate::scanners::constraints::scan_constraint::ScanConstraint;
use crate::snapshots::snapshot::Snapshot;
use crate::snapshots::snapshot_sub_region::SnapshotSubRegion;
use crate::snapshots::snapshot_region::SnapshotRegion;
use futures::future::join_all;
use squalr_engine_common::conversions::value_to_metric_size;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::tasks::trackable_task::TrackableTask;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tokio::task::JoinHandle;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

pub struct ManualScanner;

impl ManualScanner {
    const NAME: &'static str = "Manual Scan";

    pub fn scan(snapshot: Arc<RwLock<Snapshot>>, constraint: &ScanConstraint, task_identifier: Option<String>, with_logging: bool) -> Arc<TrackableTask<()>> {
        let task = TrackableTask::<()>::create(
            ManualScanner::NAME.to_string(),
            task_identifier,
        );
        let task_handle: JoinHandle<()> = tokio::spawn({
            let task = task.clone();
            let snapshot = snapshot.clone();
            let constraint = constraint.clone();
            async move {
                Self::scan_task(
                    snapshot,
                    &constraint,
                    task.get_progress_sender().clone(),
                    task.get_cancellation_token(),
                    with_logging
                ).await;

                task.complete(());
            }
        });

        task.add_handle(task_handle);
        return task;
    }

    async fn scan_task(
        snapshot: Arc<RwLock<Snapshot>>,
        constraint: &ScanConstraint,
        progress_sender: broadcast::Sender<f32>,
        cancellation_token: CancellationToken,
        with_logging: bool,
    ) {
        let region_count;
        let snapshot_regions;

        // Lock the snapshot briefly to extract the regions.
        {
            let mut snapshot = snapshot.write().unwrap();
            snapshot.sort_regions_for_scans();
            region_count = snapshot.get_region_count();
            snapshot_regions = snapshot.get_snapshot_regions();
        }

        if with_logging {
            Logger::get_instance().log(LogLevel::Info, "Performing manual scan...", None);
        }

        let start_time = Instant::now();
        let processed_region_count = Arc::new(AtomicUsize::new(0));

        let results: Vec<Arc<RwLock<SnapshotRegion>>> = join_all(snapshot_regions.iter().map(|region| {
            let processed_region_count = processed_region_count.clone();
            let progress_sender = progress_sender.clone();
            let cancellation_token = cancellation_token.clone();
            let constraint = constraint.clone();
            let region = region.clone();
            let constraint = constraint.clone_and_resolve_auto_alignment();

            tokio::spawn(async move {
                if cancellation_token.is_cancelled() {
                    return region;
                }

                if !region.read().unwrap().can_compare_with_constraint(&constraint) {
                    return region.clone();
                }

                {
                    let mut region = region.write().unwrap();
                    region.set_alignment(constraint.get_alignment());
                }

                let scan_dispatcher = ScanDispatcher::get_instance();
                let scan_dispatcher = scan_dispatcher.read().unwrap();
                let scan_results = scan_dispatcher.dispatch_scan(region.clone(), &constraint);
                
                {
                    let mut region = region.write().unwrap();
                    region.set_snapshot_sub_regions(scan_results);
                }

                let processed = processed_region_count.fetch_add(1, Ordering::SeqCst);
                
                // To reduce performance impact, only periodically send progress updates.
                if processed % 32 == 0 {
                    let progress = (processed as f32 / region_count as f32) * 100.0;
                    let _ = progress_sender.send(progress);
                }

                return region.clone();
            })
        })).await.into_iter().filter_map(Result::ok).collect();

        // Lock the snapshot briefly to update it.
        {
            let mut snapshot = snapshot.write().unwrap();
            let collected_regions = results;
            
            snapshot.set_snapshot_regions(collected_regions);
            snapshot.set_name(ManualScanner::NAME.to_string());
        }

        let duration = start_time.elapsed();
        let element_count = snapshot.read().unwrap().get_element_count(constraint.get_alignment(), constraint.get_element_type().size_in_bytes());
        let byte_count = snapshot.read().unwrap().get_byte_count();
        
        Logger::get_instance().log(LogLevel::Info, &format!("Scan complete in: {:?}", duration), None);
        Logger::get_instance().log(LogLevel::Info, &format!("Results: {} ({} bytes)", element_count, value_to_metric_size(byte_count)), None);
    }
}
