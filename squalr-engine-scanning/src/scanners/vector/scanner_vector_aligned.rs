use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::comparers::vector::scanner_vector_comparer::ScannerVectorComparer;
use crate::scanners::encoders::vector::encoder::scanner_vector_encoder::ScannerVectorEncoder;
use crate::scanners::encoders::vector::types::simd_type::SimdType;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::scanners::snapshot_scanner::Scanner;
use crate::snapshots::snapshot_region::SnapshotRegion;
use std::marker::PhantomData;
use std::simd::cmp::SimdPartialEq;
use std::simd::{LaneCount, Simd, SupportedLaneCount};

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
        Self { _marker: PhantomData }
    }
}

impl<T: SimdType + Send + Sync + PartialEq, const N: usize> Scanner for ScannerVectorAligned<T, N>
where
    LaneCount<N>: SupportedLaneCount,
    LaneCount<{ N / 2 }>: SupportedLaneCount,
    LaneCount<{ N / 4 }>: SupportedLaneCount,
    LaneCount<{ N / 8 }>: SupportedLaneCount,
    Simd<T, N>: SimdPartialEq,
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
        let encoder = ScannerVectorEncoder::<T, N>::new();
        let vector_comparer = ScannerVectorComparer::<T, N>::new();
        let simd_all_true_mask = Simd::<u8, N>::splat(0xFF);

        let results = encoder.encode(
            snapshot_region.get_current_values_pointer(&snapshot_region_filter),
            snapshot_region.get_previous_values_pointer(&snapshot_region_filter),
            scan_parameters,
            scan_filter_parameters,
            snapshot_region_filter.get_base_address(),
            snapshot_region_filter.get_region_size(),
            &vector_comparer,
            simd_all_true_mask,
        );

        return results;
    }
}
