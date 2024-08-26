use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::comparers::snapshot_scanner::Scanner;
use crate::scanners::comparers::vector::encoder::scanner_vector_comparer::ScannerVectorComparer;
use crate::scanners::comparers::vector::encoder::scanner_vector_encoder::ScannerVectorEncoder;
use crate::scanners::comparers::vector::types::simd_type::SimdType;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::snapshots::snapshot_region::SnapshotRegion;
use std::simd::{LaneCount, Simd, SupportedLaneCount};
use std::simd::prelude::SimdPartialEq;
use std::marker::PhantomData;

pub struct ScannerVectorAlignedChunked<T: SimdType + Send + Sync, const N: usize>
where
    LaneCount<N>: SupportedLaneCount,
    LaneCount<{ N / 2 }>: SupportedLaneCount,
    LaneCount<{ N / 4 }>: SupportedLaneCount,
    LaneCount<{ N / 8 }>: SupportedLaneCount,
    Simd<T, N>: SimdPartialEq,
{
    _marker: PhantomData<T>,
}

impl<T: SimdType + Send + Sync, const N: usize> ScannerVectorAlignedChunked<T, N>
where
    LaneCount<N>: SupportedLaneCount,
    LaneCount<{ N / 2 }>: SupportedLaneCount,
    LaneCount<{ N / 4 }>: SupportedLaneCount,
    LaneCount<{ N / 8 }>: SupportedLaneCount,
    Simd<T, N>: SimdPartialEq,
{
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: SimdType + Send + Sync + PartialEq, const N: usize> Scanner for ScannerVectorAlignedChunked<T, N>
where
    LaneCount<N>: SupportedLaneCount,
    LaneCount<{ N / 2 }>: SupportedLaneCount,
    LaneCount<{ N / 4 }>: SupportedLaneCount,
    LaneCount<{ N / 8 }>: SupportedLaneCount,
    Simd<T, N>: SimdPartialEq,
{
    fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_parameters: &ScanParameters,
        scan_filter_parameters: &ScanFilterParameters,
    ) -> Vec<SnapshotRegionFilter> {
        let current_value_pointer = snapshot_region.get_current_values_pointer(&snapshot_region_filter);
        let previous_value_pointer = snapshot_region.get_previous_values_pointer(&snapshot_region_filter);
        let data_type = scan_filter_parameters.get_data_type();
        let data_type_size = data_type.get_size_in_bytes();
        let memory_alignment = scan_filter_parameters.get_memory_alignment_or_default();
        let element_count = snapshot_region_filter.get_element_count(memory_alignment, data_type_size) as usize;
        let vector_comparer = ScannerVectorComparer::<T, N>::new();

        let current_values_slice = unsafe {
            std::slice::from_raw_parts(current_value_pointer, element_count * memory_alignment as usize)
        };
        let previous_values_slice = unsafe {
            std::slice::from_raw_parts(previous_value_pointer, element_count * memory_alignment as usize)
        };

        // Experimentally 1MB seemed to be the optimal chunk size on my CPU to keep all threads busy
        let chunk_size = 1 << 20;
        let num_chunks = (element_count + chunk_size - 1) / chunk_size;

        let mut new_snapshot_region_filters: Vec<SnapshotRegionFilter> = (0..num_chunks)
            .into_par_iter()
            .map(|chunk_index| {
                let first_element_index = (chunk_index * chunk_size) as u64;
                let last_element_index = ((chunk_index + 1) * chunk_size).min(element_count) as u64;
                let chunk_address_offset = first_element_index * memory_alignment as u64;
                let encoder = ScannerVectorEncoder::<T, N>::new();
                let base_address = snapshot_region_filter.get_base_address() + chunk_address_offset;
                let simd_all_true_mask = Simd::<u8, N>::splat(0xFF);

                unsafe {
                    encoder.encode(
                        current_values_slice.as_ptr().add(chunk_address_offset as usize),
                        previous_values_slice.as_ptr().add(chunk_address_offset as usize),
                        scan_parameters,
                        scan_filter_parameters,
                        base_address,
                        last_element_index - first_element_index,
                        &vector_comparer,
                        simd_all_true_mask,
                    )
                }
            })
            .reduce_with(|mut collection_a, collection_b| {
                collection_a.extend(collection_b);
                collection_a
            })
            .unwrap_or_else(Vec::new);

        // Merge adjacent regions directly within the new_snapshot_region_filters vector to avoid unnecessary reallocations.
        if !new_snapshot_region_filters.is_empty() {
            new_snapshot_region_filters.sort_by(|a, b| a.get_base_address().cmp(&b.get_base_address()));

            let mut filter_index = 0;
            while filter_index < new_snapshot_region_filters.len() - 1 {
                let (left, right) = new_snapshot_region_filters.split_at_mut(filter_index + 1);
                let current_region = &mut left[filter_index];
                let next_region = &right[0];

                if current_region.get_end_address() == next_region.get_base_address() {
                    current_region.set_end_address(next_region.get_end_address());
                    new_snapshot_region_filters.remove(filter_index + 1);
                } else {
                    filter_index += 1;
                }
            }
        }

        return new_snapshot_region_filters;
    }
}
