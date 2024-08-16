use crate::snapshots::snapshot::Snapshot;
use squalr_engine_common::conversions::value_to_metric_size;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::tasks::trackable_task::TrackableTask;
use squalr_engine_processes::process_info::ProcessInfo;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Instant;

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

        let task_clone = task.clone();
        let process_info_clone = process_info.clone();
        let snapshot_clone = snapshot.clone();

        std::thread::spawn(move || {
            Self::collect_values_task(
                process_info_clone,
                snapshot_clone,
                with_logging,
                task_clone.clone(), // Pass the cloned task to update progress
                task_clone.get_cancellation_token(),
            );

            task_clone.complete(());
        });

        task
    }

    fn collect_values_task(
        process_info: Arc<ProcessInfo>,
        snapshot: Arc<RwLock<Snapshot>>,
        with_logging: bool,
        task: Arc<TrackableTask<()>>, // Pass the task itself
        cancellation_token: Arc<AtomicBool>,
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

        snapshot_regions.par_iter().for_each(|region| {
            if cancellation_token.load(Ordering::SeqCst) {
                return;
            }

            let mut region = region.write().unwrap();

            // Attempt to read new (or initial) memory values.
            if region.read_all_memory(process_info.handle).is_ok() {
                let processed = processed_region_count.fetch_add(1, Ordering::SeqCst);
                if processed % 32 == 0 {
                    let progress = (processed as f32 / region_count as f32) * 100.0;
                    task.set_progress(progress); // Use set_progress to update progress
                }
            }
            // Else, memory region was probably deallocated. It happens, ignore it.
        });

        let duration = start_time.elapsed();
        let byte_count = snapshot.read().unwrap().get_byte_count();

        if with_logging {
            Logger::get_instance().log(LogLevel::Info, &format!("Values collected in: {:?}", duration), None);
            Logger::get_instance().log(LogLevel::Info, &format!("{} bytes read ({})", byte_count, value_to_metric_size(byte_count)), None);
        }
    }
}
