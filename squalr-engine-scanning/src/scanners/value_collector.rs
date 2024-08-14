use crate::snapshots::snapshot::Snapshot;
use futures::future::join_all;
use squalr_engine_common::conversions::value_to_metric_size;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use squalr_engine_common::tasks::trackable_task::TrackableTask;
use squalr_engine_processes::process_info::ProcessInfo;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Instant;
use tokio::task::JoinHandle;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

pub struct ValueCollector;

impl ValueCollector {
    const NAME: &'static str = "Value Collector";

    pub fn collect_values(
        process_info: ProcessInfo,
        snapshot: Arc<RwLock<Snapshot>>,
        task_identifier: Option<String>,
        with_logging: bool,
    ) -> Arc<TrackableTask<()>> {
        
        let process_info = Arc::new(process_info);
        let task = TrackableTask::<()>::create(
            ValueCollector::NAME.to_string(),
            task_identifier,
        );
        let task_handle: JoinHandle<()> = tokio::spawn({
            let task = task.clone();
            let process_info = process_info.clone();
            let snapshot = snapshot.clone();
            async move {
                Self::collect_values_task(
                    process_info,
                    snapshot,
                    with_logging,
                    task.get_progress_sender().clone(),
                    task.get_cancellation_token(),
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

        {
            let mut snapshot = snapshot.write().unwrap();
            snapshot.sort_regions_for_scans();
            region_count = snapshot.get_region_count();
            snapshot_regions = snapshot.get_snapshot_regions();
        }

        if with_logging {
            Logger::get_instance().log(LogLevel::Info, "Reading values from memory...", None);
        }

        let start_time = Instant::now();
        let processed_region_count = Arc::new(AtomicUsize::new(0));

        let results: Vec<JoinHandle<()>> = snapshot_regions.iter().map(|region| {
            let processed_region_count = processed_region_count.clone();
            let progress_sender = progress_sender.clone();
            let cancellation_token = cancellation_token.clone();
            let process_info = process_info.clone();
            let region = Arc::clone(region);  // Clone the Arc to keep using it in the iteration

            tokio::spawn(async move {
                if cancellation_token.is_cancelled() {
                    return;
                }

                let mut region = region.write().unwrap();

                // Attempt to read new (or initial) memory values.
                match region.read_all_memory(process_info.handle) {
                    Ok(_) => {
                        let processed = processed_region_count.fetch_add(1, Ordering::SeqCst);
                        if processed % 32 == 0 {
                            let progress = (processed as f32 / region_count as f32) * 100.0;
                            let _ = progress_sender.send(progress);
                        }
                    },
                    Err(_) => {
                        // Memory region was probably deallocated. It happens, ignore it.
                    },
                }
            })
        }).collect();

        join_all(results).await;

        let duration = start_time.elapsed();
        let byte_count;

        {
            let mut snapshot = snapshot.write().unwrap();
            byte_count = snapshot.get_byte_count();
            snapshot.update_element_and_byte_counts(MemoryAlignment::Alignment1, 1);
        }

        if with_logging {
            Logger::get_instance().log(LogLevel::Info, &format!("Values collected in: {:?}", duration), None);
            Logger::get_instance().log(LogLevel::Info, &format!("{} bytes read ({})", byte_count, value_to_metric_size(byte_count)), None);
        }
    }
}
