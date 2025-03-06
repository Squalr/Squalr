use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::encoders::vector::scanner_vector_encoder_cascading_periodic::ScannerVectorEncoderCascadingPeriodic;
use crate::scanners::snapshot_scanner::Scanner;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_common::structures::data_types::comparisons::vector_compare::VectorCompare;
use squalr_engine_common::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_common::structures::memory_alignment::MemoryAlignment;
use squalr_engine_common::structures::scanning::scan_compare_type::ScanCompareType;
use squalr_engine_common::structures::scanning::scan_parameters::ScanParameters;
use std::simd::{LaneCount, Simd, SupportedLaneCount};

pub struct ScannerVectorCascading<const N: usize>
where
    LaneCount<N>: SupportedLaneCount + VectorCompare<N>, {}

/// Cascading scans are the single most complex case to handle due to the base addresses not being aligned.
/// It turns out that this problem has been extensively researched under "string search algorithms".
///
/// However, we want to avoid falling back onto a generic search function if we can avoid it. We can pre-analyze the
/// scan data to use more efficient implementations when possible.
///
/// There may be a ton of sub-cases, and this may best be handled by reducing the problem to a several specialized cases.
impl<const N: usize> Scanner for ScannerVectorCascading<N>
where
    LaneCount<N>: SupportedLaneCount + VectorCompare<N>,
{
    fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_parameters: &ScanParameters,
        data_type: &DataTypeRef,
        _: MemoryAlignment,
    ) -> Vec<SnapshotRegionFilter> {
        let simd_all_true_mask = Simd::<u8, N>::splat(0xFF);
        let results;

        // For immediate comparisons, we can use a cascading periodic scan.
        match scan_parameters.get_compare_type() {
            ScanCompareType::Immediate(_scan_compare_type_immediate) => {
                let vector_encoder = ScannerVectorEncoderCascadingPeriodic::<N>::new();

                results = vector_encoder.vector_encode(
                    snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter),
                    snapshot_region.get_previous_values_filter_pointer(&snapshot_region_filter),
                    scan_parameters,
                    data_type,
                    snapshot_region_filter.get_base_address(),
                    snapshot_region_filter.get_region_size(),
                    simd_all_true_mask,
                );
            }
            ScanCompareType::Relative(_scan_compare_type_relative) => {
                panic!("Relative cascading scans are not implemented yet");
            }
            ScanCompareType::Delta(_scan_compare_type_delta) => {
                panic!("Delta cascading scans are not implemented yet");
            }
        }

        results
    }
}
