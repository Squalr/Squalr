use crate::scanners::scan_execution_context::ScanExecutionContext;
use crate::scanners::snapshot_region_memory_reader::SnapshotRegionMemoryReader;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::iter::ParallelIterator;
use squalr_engine_api::conversions::storage_size_conversions::StorageSizeConversions;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

pub struct ValueCollector;

/// Implementation of a task that collects new or initial values for the provided snapshot.
impl ValueCollector {
    pub fn collect_values(
        process_info: OpenedProcessInfo,
        snapshot: Arc<RwLock<Snapshot>>,
        with_logging: bool,
        scan_execution_context: &ScanExecutionContext,
    ) {
        let process_info = Arc::new(process_info);

        Self::collect_values_internal(process_info, snapshot, with_logging, scan_execution_context);
    }

    fn collect_values_internal(
        process_info: Arc<OpenedProcessInfo>,
        snapshot: Arc<RwLock<Snapshot>>,
        with_logging: bool,
        scan_execution_context: &ScanExecutionContext,
    ) {
        if with_logging {
            log::info!("Reading values from memory (process {})...", process_info.get_process_id_raw());
        }

        let mut snapshot = match snapshot.write() {
            Ok(guard) => guard,
            Err(error) => {
                if with_logging {
                    log::error!("Failed to acquire write lock on snapshot: {}", error);
                }

                return;
            }
        };

        let start_time = Instant::now();
        let processed_region_count = Arc::new(AtomicUsize::new(0));
        let total_region_count = snapshot.get_region_count();

        let snapshot_regions = snapshot.get_snapshot_regions_mut();

        let read_memory_iterator = |snapshot_region: &mut SnapshotRegion| {
            if scan_execution_context.should_cancel() {
                return;
            }

            // Attempt to read new (or initial) memory values. Ignore failed regions, as these are generally just deallocated pages.
            // JIRA: We probably want some way of tombstoning deallocated pages.
            let _result = snapshot_region.read_all_memory(&process_info, scan_execution_context);

            // Report progress periodically (not every time for performance)
            let processed = processed_region_count.fetch_add(1, Ordering::SeqCst);

            if processed % 32 == 0 {
                let progress = (processed as f32 / total_region_count as f32) * 100.0;
                scan_execution_context.report_progress(progress);
            }
        };

        // Collect values for each snapshot region in parallel.
        snapshot_regions.par_iter_mut().for_each(read_memory_iterator);

        if with_logging {
            let duration = start_time.elapsed();
            let byte_count = snapshot.get_byte_count();

            log::info!("Values collected in: {:?}", duration);
            log::info!(
                "{} bytes read ({})",
                byte_count,
                StorageSizeConversions::value_to_metric_size(byte_count as u128)
            );
        }
    }
}
