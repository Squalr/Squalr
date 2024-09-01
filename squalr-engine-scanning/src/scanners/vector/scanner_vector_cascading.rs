use crate::results::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::comparers::vector::scanner_vector_comparer::ScannerVectorComparer;
use crate::scanners::encoders::vector::scanner_vector_encoder_cascading_periodic::ScannerVectorEncoderCascadingPeriodic;
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
        let results;

        // For immediate comparisons, we can use a cascading periodic scan
        if scan_parameters.is_immediate_comparison() {
            let vector_comparer = ScannerVectorComparer::<T, N>::new();

            results = encoder.encode(
                snapshot_region.get_current_values_pointer(&snapshot_region_filter),
                snapshot_region.get_previous_values_pointer(&snapshot_region_filter),
                scan_parameters,
                scan_filter_parameters,
                snapshot_region_filter.get_base_address(),
                snapshot_region_filter.get_region_size(),
                &vector_comparer,
                simd_all_true_mask,
            );
        } else {
            panic!("relative and delta cascading scans are not implemented yet");
        }

        return results;
    }
}
