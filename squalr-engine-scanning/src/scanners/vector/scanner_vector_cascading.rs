use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::comparers::vector::scanner_vector_comparer_cascading::ScannerVectorComparerCascading;
use crate::scanners::encoders::vector::scanner_vector_encoder::ScannerVectorEncoder;
use crate::scanners::encoders::vector::simd_type::SimdType;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::scanners::snapshot_scanner::Scanner;
use crate::snapshots::snapshot_region::SnapshotRegion;
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

/// A special scan that supports scanning for data types that are larger than the given memory alignment.
/// This works by cascading scans together using OR operations.
/// In this case, the scan results are no longer independent variables, which means contiguous scan reslts need to be combined.
/// See ScannerVectorComparerCascading for the implementation of the cascading OR combinations.
/// Additionally, ScannerVectorEncoder has some logic to prevent over-reading into adjacent memory by padding each scan by 1 element.
impl<T: SimdType + Send + Sync + PartialEq, const N: usize> Scanner for ScannerVectorCascading<T, N>
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
        let memory_alignment = scan_filter_parameters.get_memory_alignment_or_default();
        let encoder = ScannerVectorEncoder::<T, N>::new();
        let vector_comparer = ScannerVectorComparerCascading::<T, N>::new(memory_alignment);
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
