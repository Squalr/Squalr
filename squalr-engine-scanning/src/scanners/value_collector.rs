use crate::results::scan_results::ScanResults;
use crate::snapshots::snapshot_region::SnapshotRegion;
use crate::snapshots::snapshot::Snapshot;
use squalr_engine_common::conversions::value_to_metric_size;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::tasks::trackable_task::TrackableTask;
use squalr_engine_memory::memory_reader::memory_reader_trait::IMemoryReader;
use squalr_engine_memory::memory_reader::MemoryReader;
use squalr_engine_processes::process_info::ProcessInfo;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::borrow::{Borrow, BorrowMut};
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
        let memory_pages = scan_results.get_memory_pages_for_scan(&process_info.borrow());
        let total_region_count = memory_pages.len();

        let start_time = Instant::now();
        let processed_region_count = Arc::new(AtomicUsize::new(0));

        let snapshot_regions: Vec<SnapshotRegion> = memory_pages
        .into_par_iter()
        .filter_map(|memory_page| {
            if cancellation_token.load(Ordering::SeqCst) {
                return None;
            }
            
            // Attempt to read new (or initial) memory values.
            let mut bytes = vec![0u8; memory_page.get_region_size() as usize];
            let result = MemoryReader::get_instance().read_bytes(process_info.handle, memory_page.get_base_address(), bytes.borrow_mut());
    
            // Report progress periodically (not every time for performance)
            let processed = processed_region_count.fetch_add(1, Ordering::SeqCst);
            if processed % 32 == 0 {
                let progress = (processed as f32 / total_region_count as f32) * 100.0;
                task.set_progress(progress);
            }
            
            // Page was likely deallocated
            if result.is_err() {
                return None;
            }
    
            Some(SnapshotRegion::new(memory_page, bytes))
        })
        .collect();
    

        let snapshot = Arc::new(RwLock::new(Snapshot::new(Self::NAME.to_string(), snapshot_regions)));

        scan_results.set_snapshot(snapshot.clone());

        if with_logging {
            let duration = start_time.elapsed();
            let byte_count = snapshot.read().unwrap().get_byte_count();

            Logger::get_instance().log(LogLevel::Info, &format!("Values collected in: {:?}", duration), None);
            Logger::get_instance().log(LogLevel::Info, &format!("{} bytes read ({})", byte_count, value_to_metric_size(byte_count)), None);
        }
    }
}
