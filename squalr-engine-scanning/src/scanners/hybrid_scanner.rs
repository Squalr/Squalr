use crate::scanners::comparers::scan_dispatcher::ScanDispatcher;
use crate::scanners::constraints::scan_constraint::ScanConstraint;
use crate::snapshots::snapshot::Snapshot;
use crate::snapshots::snapshot_sub_region::SnapshotSubRegion;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
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

    pub fn scan(process_info: ProcessInfo, snapshot: Arc<RwLock<Snapshot>>, constraint: &ScanConstraint, task_identifier: Option<String>, with_logging: bool) -> Arc<TrackableTask<()>> {
        let task = TrackableTask::<()>::create(
            HybridScanner::NAME.to_string(),
            task_identifier,
        );

        let task_clone = task.clone();
        let constraint_clone = constraint.clone();

        thread::spawn(move || {
            Self::scan_task(
                process_info,
                snapshot,
                &constraint_clone,
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
        constraint: &ScanConstraint,
        task: Arc<TrackableTask<()>>,
        cancellation_token: Arc<AtomicBool>,
        with_logging: bool,
    ) {
        let mut snapshot = snapshot.write().unwrap();
        let constraint = &constraint.clone_and_resolve_auto_alignment();
        let region_count = snapshot.get_region_count();
        let snapshot_regions = snapshot.get_snapshot_regions_mut();

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

                // Attempt to read new (or initial) memory values.
                if !region.read_all_memory_parallel(process_info.handle).is_ok() {
                    // Memory region was probably deallocated. It happens, ignore it.
                    region.set_snapshot_sub_regions(vec![]);
                    return;
                }

                // Create the default sub-region if it does not exist yet
                if region.get_snapshot_sub_regions().is_empty() && region.get_region_size() > 0 {
                    region.set_snapshot_sub_regions(vec![SnapshotSubRegion::new(&region)]);
                }

                if !region.can_compare_with_constraint(constraint) {
                    processed_region_count.fetch_add(1, Ordering::SeqCst);
                    return;
                }

                region.set_alignment(constraint.get_alignment());

                let scan_dispatcher = ScanDispatcher::get_instance();
                let scan_results = scan_dispatcher.dispatch_scan_parallel(region, constraint);

                region.set_snapshot_sub_regions(scan_results.to_owned());

                let processed = processed_region_count.fetch_add(1, Ordering::SeqCst);

                // To reduce performance impact, only periodically send progress updates.
                if processed % 32 == 0 {
                    let progress = (processed as f32 / region_count as f32) * 100.0;
                    task.set_progress(progress);
                }
            });
        
        snapshot.set_name(HybridScanner::NAME.to_string());

        let duration = start_time.elapsed();
        let element_count = snapshot.get_element_count(constraint.get_alignment(), constraint.get_data_type().size_in_bytes());
        let byte_count = snapshot.get_byte_count();

        Logger::get_instance().log(LogLevel::Info, &format!("Scan complete in: {:?}", duration), None);
        Logger::get_instance().log(LogLevel::Info, &format!("Results: {} ({} bytes)", element_count, value_to_metric_size(byte_count)), None);
    }
}
