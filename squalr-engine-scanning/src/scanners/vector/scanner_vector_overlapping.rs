use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::snapshot_scanner::Scanner;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_api::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use squalr_engine_api::structures::scanning::scan_parameters_local::ScanParametersLocal;
use squalr_engine_api::structures::{data_types::generics::vector_comparer::VectorComparer, scanning::comparisons::scan_compare_type::ScanCompareType};
use std::simd::{LaneCount, Simd, SupportedLaneCount};

pub struct ScannerVectorOverlapping<const N: usize>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>, {}

/// Overlapping scans are the single most complex case to handle due to the base addresses not being aligned.
/// It turns out that this problem has been extensively researched under "string search algorithms".
///
/// However, we want to avoid falling back onto a generic search function if we can avoid it. We can pre-analyze the
/// scan data to use more efficient implementations when possible.
///
/// There may be a ton of sub-cases, and this may best be handled by reducing the problem to a several specialized cases.
impl<const N: usize> Scanner for ScannerVectorOverlapping<N>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
{
    fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Vec<SnapshotRegionFilter> {
        let simd_all_true_mask = Simd::<u8, N>::splat(0xFF);
        let results;

        // For immediate comparisons, we can use a overlapping periodic scan.
        match scan_parameters_global.get_compare_type() {
            ScanCompareType::Immediate(_scan_compare_type_immediate) => {
                panic!("Relative overlapping scans are not implemented yet");
            }
            ScanCompareType::Relative(_scan_compare_type_relative) => {
                panic!("Relative overlapping scans are not implemented yet");
            }
            ScanCompareType::Delta(_scan_compare_type_delta) => {
                panic!("Delta overlapping scans are not implemented yet");
            }
        }

        results
    }
}
