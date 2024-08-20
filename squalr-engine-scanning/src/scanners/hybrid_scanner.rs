use crate::scanners::comparers::scan_dispatcher::ScanDispatcher;
use crate::scanners::constraints::scan_constraint::ScanConstraint;
use crate::snapshots::snapshot::Snapshot;
use rayon::iter::{IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use squalr_engine_common::conversions::value_to_metric_size;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::tasks::trackable_task::TrackableTask;
use squalr_engine_processes::process_info::ProcessInfo;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use std::thread;

pub struct HybridScanner;

/// Implementation of a task that collects values and performs a constraint scan in the same thread pool. This is much faster than doing
/// ValueCollector => ManualScanner, however this means that regions processed last will not have their values collected until potentially
/// much later than the scan was initiated.
impl HybridScanner {
    const NAME: &'static str = "Hybrid Scan";

    pub fn scan(
        process_info: ProcessInfo,
        snapshot: Arc<RwLock<Snapshot>>,
        scan_constrant: &ScanConstraint,
        task_identifier: Option<String>,
        with_logging: bool,
    ) -> Arc<TrackableTask<()>> {
        let task = TrackableTask::<()>::create(
            HybridScanner::NAME.to_string(),
            task_identifier,
        );

        let task_clone = task.clone();
        let scan_constrant_clone = scan_constrant.clone();

        thread::spawn(move || {
            Self::scan_task(
                process_info,
                snapshot,
                &scan_constrant_clone,
                task_clone.clone(),
                task_clone.get_cancellation_token().clone(),
                with_logging
            );

            task_clone.complete(());
        });

        return task;
    }

    fn scan_task(
        process_info: ProcessInfo,
        snapshot: Arc<RwLock<Snapshot>>,
        scan_constrant: &ScanConstraint,
        task: Arc<TrackableTask<()>>,
        cancellation_token: Arc<AtomicBool>,
        with_logging: bool,
    ) {
        let mut snapshot = snapshot.write().unwrap();

        snapshot.initialize_for_constraint(scan_constrant);

        let scan_constrant = &scan_constrant.clone();
        let region_count = snapshot.get_region_count();
        let snapshot_regions = snapshot.get_snapshot_regions_for_update();

        if with_logging {
            Logger::get_instance().log(LogLevel::Info, "Performing hybrid manual scan...", None);
        }

        let start_time = Instant::now();
        let processed_region_count = Arc::new(AtomicUsize::new(0));

        snapshot_regions
            .par_iter_mut()
            .for_each(|region| {
                if cancellation_token.load(Ordering::SeqCst) {
                    return;
                }

                // Attempt to read new (or initial) memory values. Ignore failures as they usually indicate deallocated pages.
                let _ = region.read_all_memory_parallel(process_info.handle);

                // Create filters for the constraint
                region.create_filters_for_constraint(scan_constrant);
                let snapshot_region_filters = region.get_filters();

                // Perform scan using the ScanDispatcher
                let results = snapshot_region_filters
                    .into_par_iter()
                    .filter_map(|(data_type, snapshot_region_filter)| {
                        if cancellation_token.load(Ordering::SeqCst) {
                            return None;
                        }

                        let scan_dispatcher = ScanDispatcher::get_instance();
                        let scan_results = scan_dispatcher.dispatch_scan_parallel(
                            region,
                            snapshot_region_filter,
                            scan_constrant,
                            data_type,
                        );

                        Some((data_type.clone(), scan_results))
                    })
                    .collect();

                region.set_all_filters(results);

                let processed = processed_region_count.fetch_add(1, Ordering::SeqCst);

                // To reduce performance impact, only periodically send progress updates.
                if processed % 32 == 0 {
                    let progress = (processed as f32 / region_count as f32) * 100.0;
                    task.set_progress(progress);
                }
            });

        let duration = start_time.elapsed();
        let byte_count = snapshot.get_byte_count();

        if with_logging {
            Logger::get_instance().log(LogLevel::Info, &format!("Scan complete in: {:?}", duration), None);
            Logger::get_instance().log(LogLevel::Info, &format!("{} bytes read ({})", byte_count, value_to_metric_size(byte_count)), None);
        }
    }
}
