use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::encoders::vector::scanner_vector_encoder::ScannerVectorEncoder;
use crate::scanners::snapshot_scanner::Scanner;
use crate::snapshots::snapshot_region::SnapshotRegion;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use squalr_engine_common::structures::scanning::scan_parameters::ScanParameters;
use squalr_engine_common::structures::{data_types::comparisons::vector_compare::VectorCompare, scanning::scan_filter_parameters::ScanFilterParameters};
use std::simd::{LaneCount, Simd, SupportedLaneCount};

pub struct ScannerVectorAlignedChunked<const N: usize>
where
    LaneCount<N>: SupportedLaneCount + VectorCompare<N>, {}

impl<const N: usize> Scanner for ScannerVectorAlignedChunked<N>
where
    LaneCount<N>: SupportedLaneCount + VectorCompare<N>,
{
    /// Performs a sequential iteration over a region of memory, performing the scan comparison.
    /// A run-length encoding algorithm is used to generate new sub-regions as the scan progresses.
    fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_parameters: &ScanParameters,
        scan_filter_parameters: &ScanFilterParameters,
    ) -> Vec<SnapshotRegionFilter> {
        let vector_encoder = ScannerVectorEncoder::<N>::new();
        let simd_all_true_mask = Simd::<u8, N>::splat(0xFF);
        let region_size = snapshot_region_filter.get_region_size();
        let chunk_size = 1024 * 1024 * 1; // 1 MB
        let num_chunks = (region_size + chunk_size - 1) / chunk_size;

        let mut results: Vec<SnapshotRegionFilter> = (0..num_chunks)
            .into_par_iter()
            .map(|chunk_index| {
                let chunk_start_offset_bytes = (chunk_index * chunk_size) as u64;
                let chunk_start_address = snapshot_region_filter.get_base_address() + chunk_start_offset_bytes;
                let current_value_pointer = snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter);
                let previous_value_pointer = snapshot_region.get_previous_values_filter_pointer(&snapshot_region_filter);
                let remaining_size = region_size as u64 - chunk_start_offset_bytes;
                let chunk_region_size = remaining_size.min(chunk_size as u64);

                vector_encoder.vector_encode(
                    unsafe { current_value_pointer.add(chunk_start_offset_bytes as usize) },
                    unsafe { previous_value_pointer.add(chunk_start_offset_bytes as usize) },
                    scan_parameters,
                    scan_filter_parameters.get_data_type(),
                    chunk_start_address,
                    chunk_region_size,
                    simd_all_true_mask,
                )
            })
            .reduce_with(|mut collection_a, collection_b| {
                collection_a.extend(collection_b);
                collection_a
            })
            .unwrap_or_else(Vec::new);

        // Merge adjacent regions directly within the new_snapshot_region_filters vector to avoid unnecessary reallocations.
        if !results.is_empty() {
            // Ensure that filters are sorted by base address ascending.
            results.sort_by(|a, b| a.get_base_address().cmp(&b.get_base_address()));

            let mut filter_index = 0;
            while filter_index < results.len() - 1 {
                let (left, right) = results.split_at_mut(filter_index + 1);
                let current_region = &mut left[filter_index];
                let next_region = &right[0];

                if current_region.get_end_address() == next_region.get_base_address() {
                    current_region.set_end_address(next_region.get_end_address());
                    // Remove the next region as it has been merged.
                    results.remove(filter_index + 1);
                } else {
                    filter_index += 1;
                }
            }
        }

        results
    }
}
