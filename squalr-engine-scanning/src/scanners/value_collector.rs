use crate::results::scan_results::ScanResults;
use crate::snapshots::snapshot_sub_region::SnapshotSubRegion;
use crate::snapshots::snapshot::Snapshot;
use squalr_engine_common::conversions::value_to_metric_size;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::tasks::trackable_task::TrackableTask;
use squalr_engine_processes::process_info::ProcessInfo;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use std::borrow::Borrow;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Instant;

pub struct ValueCollector;

/// Implementation of a task that collects new or initial values for the provided snapshot.
impl ValueCollector {
    const NAME: &'static str = "Value Collector";

    pub fn collect_values(
        process_info: ProcessInfo,
        scan_results: Arc<RwLock<ScanResults>>,
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
        let scan_results = scan_results.clone();

        std::thread::spawn(move || {
            Self::collect_values_task(
                process_info_clone,
                scan_results,
                with_logging,
                task_clone.clone(),
                task_clone.get_cancellation_token(),
            );

            task_clone.complete(());
        });

        return task;
    }

    fn collect_values_task(
        process_info: Arc<ProcessInfo>,
        scan_results: Arc<RwLock<ScanResults>>,
        with_logging: bool,
        task: Arc<TrackableTask<()>>,
        cancellation_token: Arc<AtomicBool>,
    ) {
        if with_logging {
            Logger::get_instance().log(LogLevel::Info, "Reading values from memory...", None);
        }

        let mut scan_results = scan_results.write().unwrap();
        let snapshot = scan_results.get_snapshot_create_if_none(&process_info.borrow());
        let mut snapshot = snapshot.write().unwrap();
        let total_region_count = snapshot.get_region_count();

        let start_time = Instant::now();
        let processed_region_count = Arc::new(AtomicUsize::new(0));

        snapshot.get_snapshot_regions_mut().par_iter_mut().for_each(|region| {
            if cancellation_token.load(Ordering::SeqCst) {
                return;
            }

            // Attempt to read new (or initial) memory values. Ignore failures, as this often just means a page was deallocated.
            let _ = region.read_all_memory_parallel(process_info.handle);

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

            Logger::get_instance().log(LogLevel::Info, &format!("Values collected in: {:?}", duration), None);
            Logger::get_instance().log(LogLevel::Info, &format!("{} bytes read ({})", byte_count, value_to_metric_size(byte_count)), None);
        }
    }
}
