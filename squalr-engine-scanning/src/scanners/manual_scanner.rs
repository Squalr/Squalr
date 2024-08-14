use crate::scanners::comparers::scan_dispatcher::ScanDispatcher;
use crate::scanners::constraints::scan_constraints::ScanConstraints;
use crate::snapshots::snapshot::Snapshot;
use crate::snapshots::snapshot_element_range::SnapshotElementRange;
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

    pub fn scan(snapshot: Arc<RwLock<Snapshot>>, constraints: Arc<RwLock<ScanConstraints>>, task_identifier: Option<String>) -> Arc<TrackableTask<()>> {
        let task = TrackableTask::<()>::create(
            ManualScanner::NAME.to_string(),
            task_identifier,
        );
        let task_handle: JoinHandle<()> = tokio::spawn({
            let task = task.clone();
            let snapshot = snapshot.clone();
            let constraints = constraints.clone();
            async move {
                Self::scan_task(
                    snapshot,
                    constraints,
                    task.get_progress_sender().clone(),
                    task.get_cancellation_token(),
                ).await;

                task.complete(());
            }
        });

        task.add_handle(task_handle);
        return task;
    }

    async fn scan_task(
        snapshot: Arc<RwLock<Snapshot>>,
        constraints: Arc<RwLock<ScanConstraints>>,
        progress_sender: broadcast::Sender<f32>,
        cancellation_token: CancellationToken,
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

        let start_time = Instant::now();
        let processed_region_count = Arc::new(AtomicUsize::new(0));

        let results: Vec<Arc<RwLock<SnapshotRegion>>> = join_all(snapshot_regions.iter().map(|region| {
            let processed_region_count = processed_region_count.clone();
            let progress_sender = progress_sender.clone();
            let cancellation_token = cancellation_token.clone();
            let constraints = constraints.clone();
            let region = region.clone();

            tokio::spawn(async move {
                if cancellation_token.is_cancelled() {
                    return region;
                }
                
                let constraints = constraints.read().unwrap();
                let mut region_mut = region.write().unwrap();

                if region_mut.can_compare_with_constraints(&constraints) {
                    region_mut.set_byte_alignment(constraints.get_byte_alignment());
                    
                    let scan_dispatcher = ScanDispatcher::get_instance();
                    let scan_dispatcher = scan_dispatcher.read().unwrap();
                    let scan_results: Vec<Arc<RwLock<SnapshotElementRange>>> =scan_dispatcher.dispatch_scan(region.clone(), &constraints);
                    
                    region_mut.set_snapshot_element_ranges(scan_results);
                    region_mut.set_byte_alignment(constraints.get_byte_alignment());
                    region_mut.set_data_type_size(constraints.get_element_type().size_in_bytes());
    
                    let processed = processed_region_count.fetch_add(1, Ordering::SeqCst);
                    
                    // To reduce performance impact, only periodically send progress updates
                    if processed % 32 == 0 {
                        let progress = (processed as f32 / region_count as f32) * 100.0;
                        let _ = progress_sender.send(progress);
                    }
                }

                drop(region_mut);
                return region;
            })
        })).await.into_iter().filter_map(Result::ok).collect(); // Collecting the results

        // Lock the snapshot briefly to update it.
        {
            let mut snapshot = snapshot.write().unwrap();
            let constraints = constraints.read().unwrap();
            let collected_regions = results.into_iter().map(|r| Arc::try_unwrap(r).unwrap().into_inner().unwrap()).collect();
            
            snapshot.set_snapshot_regions(collected_regions, constraints.get_byte_alignment(), constraints.get_element_type().size_in_bytes());
            snapshot.set_name(ManualScanner::NAME.to_string());
        }

        let duration = start_time.elapsed();

        Logger::get_instance().log(LogLevel::Info, &format!("Scan complete in: {:?}", duration), None);
        let snapshot = snapshot.read().unwrap();
        Logger::get_instance().log(LogLevel::Info, &format!("Results: {} ({} bytes)", snapshot.get_element_count(), value_to_metric_size(snapshot.get_byte_count())), None);
    }
}
