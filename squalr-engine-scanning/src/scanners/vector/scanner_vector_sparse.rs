use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::encoders::vector::scanner_vector_encoder::ScannerVectorEncoder;
use crate::scanners::snapshot_scanner::Scanner;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_api::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use squalr_engine_api::structures::scanning::scan_parameters_local::ScanParametersLocal;
use squalr_engine_api::structures::{data_types::generics::vector_comparer::VectorComparer, memory::memory_alignment::MemoryAlignment};
use std::simd::{LaneCount, Simd, SupportedLaneCount};

pub struct ScannerVectorSparse<const N: usize>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>, {}

impl<const N: usize> ScannerVectorSparse<N>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
{
    // This mask automatically captures all in-between elements. For example, scanning for Byte 0 with an alignment of 2-bytes
    // against <0, 24, 0, 43> would all return true, due to this mask of <0, 255, 0, 255>. Scan results will automatically skip
    // over the unwanted elements based on alignment. In fact, we do NOT want to break this into two separate snapshot regions,
    // since this would be incredibly inefficient. So in this example, we would return a single snapshot region of size 4, and the scan results would iterate by 2.
    pub fn get_sparse_mask(memory_alignment: MemoryAlignment) -> Simd<u8, N> {
        match memory_alignment {
            // This will produce a byte pattern of <0xFF, 0xFF...>.
            MemoryAlignment::Alignment1 => Simd::<u8, N>::splat(0xFF),
            // This will produce a byte pattern of <0x00, 0xFF...>.
            MemoryAlignment::Alignment2 => {
                let mut mask = [0u8; N];
                for index in (1..N).step_by(2) {
                    mask[index] = 0xFF;
                }
                Simd::from_array(mask)
            }
            // This will produce a byte pattern of <0x00, 0x00, 0x00, 0xFF...>.
            MemoryAlignment::Alignment4 => {
                let mut mask = [0u8; N];
                for index in (3..N).step_by(4) {
                    mask[index] = 0xFF;
                }
                Simd::from_array(mask)
            }
            // This will produce a byte pattern of <0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF...>.
            MemoryAlignment::Alignment8 => {
                let mut mask = [0u8; N];
                for index in (7..N).step_by(8) {
                    mask[index] = 0xFF;
                }
                Simd::from_array(mask)
            }
        }
    }
}

impl<const N: usize> Scanner for ScannerVectorSparse<N>
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
        let vector_encoder = ScannerVectorEncoder::<N>::new();

        let results = vector_encoder.vector_encode(
            snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter),
            snapshot_region.get_previous_values_filter_pointer(&snapshot_region_filter),
            scan_parameters_global,
            scan_parameters_local,
            snapshot_region_filter.get_base_address(),
            snapshot_region_filter.get_region_size(),
            Self::get_sparse_mask(scan_parameters_local.get_memory_alignment_or_default()),
        );

        results
    }
}
