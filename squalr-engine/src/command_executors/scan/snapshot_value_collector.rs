use rayon::iter::{IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use squalr_engine_api::conversions::storage_size_conversions::StorageSizeConversions;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_session::os::engine_os_provider::MemoryReadProvider;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;

pub struct SnapshotValueCollector;

impl SnapshotValueCollector {
    pub fn collect_values(
        process_info: OpenedProcessInfo,
        snapshot: Arc<RwLock<Snapshot>>,
        memory_read_provider: Arc<dyn MemoryReadProvider>,
        with_logging: bool,
    ) {
        if with_logging {
            log::info!("Reading values from memory (process {})...", process_info.get_process_id_raw());
        }

        let mut snapshot_guard = match snapshot.write() {
            Ok(snapshot_guard) => snapshot_guard,
            Err(error) => {
                if with_logging {
                    log::error!("Failed to acquire write lock on snapshot: {}", error);
                }

                return;
            }
        };

        let start_time = Instant::now();
        let processed_region_count = Arc::new(AtomicUsize::new(0));
        let total_region_count = snapshot_guard.get_region_count();
        let process_info = Arc::new(process_info);

        snapshot_guard
            .get_snapshot_regions_mut()
            .par_iter_mut()
            .for_each(|snapshot_region| {
                Self::read_snapshot_region_values(snapshot_region, &process_info, memory_read_provider.as_ref());

                let processed_region_index = processed_region_count.fetch_add(1, Ordering::SeqCst);

                if processed_region_index % 32 == 0 && total_region_count > 0 {
                    let progress = (processed_region_index as f32 / total_region_count as f32) * 100.0;
                    log::debug!("Value collection progress: {:.1}%.", progress);
                }
            });

        if with_logging {
            let duration = start_time.elapsed();
            let byte_count = snapshot_guard.get_byte_count();

            log::info!("Values collected in: {:?}", duration);
            log::info!(
                "{} bytes read ({})",
                byte_count,
                StorageSizeConversions::value_to_metric_size(byte_count as u128)
            );
        }
    }

    fn read_snapshot_region_values(
        snapshot_region: &mut SnapshotRegion,
        process_info: &OpenedProcessInfo,
        memory_read_provider: &dyn MemoryReadProvider,
    ) {
        let region_size = snapshot_region.get_region_size() as usize;
        let base_address = snapshot_region.get_base_address();

        if region_size == 0 {
            snapshot_region.page_boundary_tombstones.insert(base_address);
            return;
        }

        std::mem::swap(&mut snapshot_region.current_values, &mut snapshot_region.previous_values);

        if snapshot_region.current_values.is_empty() {
            snapshot_region.current_values = vec![0_u8; region_size];
        }

        if snapshot_region.page_boundaries.is_empty() {
            if !memory_read_provider.read_bytes(process_info, base_address, &mut snapshot_region.current_values) {
                snapshot_region.page_boundary_tombstones.insert(base_address);
            }

            return;
        }

        let mut read_ranges = Vec::with_capacity(snapshot_region.page_boundaries.len().saturating_add(1));
        let mut next_range_start_address = base_address;
        let mut current_slice = snapshot_region.current_values.as_mut_slice();

        for &next_boundary_address in &snapshot_region.page_boundaries {
            let range_size = next_boundary_address.saturating_sub(next_range_start_address) as usize;
            let (range_slice, remaining_slice) = current_slice.split_at_mut(range_size);

            if !range_slice.is_empty() {
                read_ranges.push((next_range_start_address, range_slice));
            }

            current_slice = remaining_slice;
            next_range_start_address = next_boundary_address;
        }

        if !current_slice.is_empty() {
            read_ranges.push((next_range_start_address, current_slice));
        }

        let read_failures = read_ranges
            .into_par_iter()
            .filter_map(|(address, buffer)| {
                if memory_read_provider.read_bytes(process_info, address, buffer) {
                    None
                } else {
                    Some(address)
                }
            })
            .collect::<Vec<_>>();

        snapshot_region.page_boundary_tombstones.extend(read_failures);
    }
}
