use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::comparers::snapshot_scanner::Scanner;
use crate::scanners::comparers::vector::encoder::scanner_vector_comparer::ScannerVectorComparer;
use crate::scanners::comparers::vector::encoder::scanner_vector_encoder::ScannerVectorEncoder;
use crate::scanners::comparers::vector::types::simd_type::SimdType;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::snapshots::snapshot_region::SnapshotRegion;
use std::simd::{LaneCount, Simd, SupportedLaneCount};
use std::marker::PhantomData;

pub struct ScannerVectorAligned<T: SimdType + Send + Sync, const N: usize>
where
    LaneCount<N>: SupportedLaneCount,
{
    _marker: PhantomData<T>,
}

impl<T: SimdType + Send + Sync, const N: usize> ScannerVectorAligned<T, N>
where
    LaneCount<N>: SupportedLaneCount,
{
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: SimdType + Send + Sync, const N: usize> Scanner for ScannerVectorAligned<T, N>
where
    LaneCount<N>: SupportedLaneCount,
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
        let data_type = scan_filter_parameters.get_data_type();
        let data_type_size = data_type.get_size_in_bytes();
        let memory_alignment = scan_filter_parameters.get_memory_alignment_or_default();
        let encoder = ScannerVectorEncoder::<T, N>::new();
        let vector_comparer = ScannerVectorComparer::<T, N>::new();
        let simd_all_true_mask = Simd::<u8, N>::splat(0xFF);

        encoder.encode(
            snapshot_region.get_current_values_pointer(&snapshot_region_filter),
            snapshot_region.get_previous_values_pointer(&snapshot_region_filter),
            scan_parameters,
            scan_filter_parameters,
            snapshot_region_filter.get_base_address(),
            snapshot_region_filter.get_element_count(memory_alignment, data_type_size),
            &vector_comparer,
            simd_all_true_mask,
        )
    }
}
