use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::comparers::vector::scanner_vector_comparer::ScannerVectorComparer;
use crate::scanners::encoders::vector::scanner_vector_encoder_cascading_periodic::ScannerVectorEncoderCascadingPeriodic;
use crate::scanners::encoders::vector::simd_type::SimdType;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::scanners::snapshot_scanner::Scanner;
use crate::snapshots::snapshot_region::SnapshotRegion;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::marker::PhantomData;
use std::simd::prelude::SimdPartialEq;
use std::simd::{LaneCount, Simd, SupportedLaneCount};

pub struct ScannerVectorCascading<T: SimdType + Send + Sync, const N: usize>
where
    LaneCount<N>: SupportedLaneCount,
    LaneCount<{ N / 2 }>: SupportedLaneCount,
    LaneCount<{ N / 4 }>: SupportedLaneCount,
    LaneCount<{ N / 8 }>: SupportedLaneCount,
    Simd<T, N>: SimdPartialEq,
{
    _marker: PhantomData<T>,
}

impl<T: SimdType + Send + Sync, const N: usize> ScannerVectorCascading<T, N>
where
    LaneCount<N>: SupportedLaneCount,
    LaneCount<{ N / 2 }>: SupportedLaneCount,
    LaneCount<{ N / 4 }>: SupportedLaneCount,
    LaneCount<{ N / 8 }>: SupportedLaneCount,
    Simd<T, N>: SimdPartialEq,
{
    pub fn new() -> Self {
        Self { _marker: PhantomData }
    }
}

/// Cascading scans are the single most complex case to handle due to the base addresses not being aligned.
/// It turns out that this problem has been extensively researched under "string search algorithms".
///
/// However, we want to avoid falling back onto a generic search function if we can avoid it. We can pre-analyze the
/// scan data to use more efficient implementations when possible.
///
/// There may be a ton of sub-cases, and this may best be handled by reducing the problem to a several specialized cases.
impl<T: SimdType + Send + Sync + PartialEq, const N: usize> Scanner for ScannerVectorCascading<T, N>
where
    LaneCount<N>: SupportedLaneCount,
    LaneCount<{ N / 2 }>: SupportedLaneCount,
    LaneCount<{ N / 4 }>: SupportedLaneCount,
    LaneCount<{ N / 8 }>: SupportedLaneCount,
    Simd<T, N>: SimdPartialEq,
{
    unsafe fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_parameters: &ScanParameters,
        scan_filter_parameters: &ScanFilterParameters,
    ) -> Vec<SnapshotRegionFilter> {
        let encoder: ScannerVectorEncoderCascadingPeriodic<T, N> = ScannerVectorEncoderCascadingPeriodic::<T, N>::new();
        let simd_all_true_mask = Simd::<u8, N>::splat(0xFF);
        let mut results;

        // For immediate comparisons, we can use a cascading periodic scan
        if scan_parameters.is_immediate_comparison() {
            let vector_comparer = ScannerVectorComparer::<T, N>::new();
            let current_value_pointer = snapshot_region.get_current_values_pointer(&snapshot_region_filter);
            let previous_value_pointer = snapshot_region.get_previous_values_pointer(&snapshot_region_filter);
            let region_size = snapshot_region_filter.get_region_size();

            let chunk_size = 1024 * 1024 * 1; // 1 MB
            let num_chunks = (region_size + chunk_size - 1) / chunk_size;

            // Convert the pointers to slices
            let current_values_slice = std::slice::from_raw_parts(current_value_pointer, region_size as usize);
            let previous_values_slice = std::slice::from_raw_parts(previous_value_pointer, region_size as usize);

            results = (0..num_chunks)
                .into_par_iter()
                .map(|chunk_index| {
                    let chunk_start_address = (chunk_index * chunk_size) as u64;
                    let remaining_size = region_size as u64 - chunk_start_address;
                    let chunk_region_size = remaining_size.min(chunk_size as u64);

                    unsafe {
                        encoder.encode(
                            current_values_slice.as_ptr().add(chunk_start_address as usize),
                            previous_values_slice.as_ptr().add(chunk_start_address as usize),
                            scan_parameters,
                            scan_filter_parameters,
                            chunk_start_address,
                            chunk_region_size,
                            &vector_comparer,
                            simd_all_true_mask,
                        )
                    }
                })
                .reduce_with(|mut collection_a, collection_b| {
                    collection_a.extend(collection_b);
                    return collection_a;
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
        } else {
            panic!("relative and delta cascading scans are not implemented yet");
        }

        return results;
    }
}
