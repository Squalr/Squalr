use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::comparers::snapshot_scanner::Scanner;
use crate::scanners::comparers::vector::encoder::scanner_vector_comparer::ScannerVectorComparer;
use crate::scanners::comparers::vector::encoder::scanner_vector_encoder::ScannerVectorEncoder;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::simd::{LaneCount, Simd, SupportedLaneCount};
use std::marker::PhantomData;

use super::types::simd_type::SimdType;

pub struct ScannerVectorStaggered<T: SimdType + Send + Sync, const N: usize>
where
    LaneCount<N>: SupportedLaneCount,
{
    _marker: PhantomData<T>,
}

impl<T: SimdType + Send + Sync, const N: usize> ScannerVectorStaggered<T, N>
where
    LaneCount<N>: SupportedLaneCount,
{
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
    
    /// Generates staggered masks based on the data type size and memory alignment.
    fn get_staggered_mask(data_type_size: u64, memory_alignment: MemoryAlignment) -> Vec<Simd<u8, N>> {
        match (data_type_size, memory_alignment) {
            // Data type size 2
            (2, MemoryAlignment::Alignment1) => vec![
                {
                    let mut mask = [0u8; N];
                    for index in (0..N).step_by(2) {
                        mask[index] = 0xFF;
                    }
                    Simd::from_array(mask)
                },
                {
                    let mut mask = [0u8; N];
                    for index in (1..N).step_by(2) {
                        mask[index] = 0xFF;
                    }
                    Simd::from_array(mask)
                },
            ],
            // Data type size 4
            (4, MemoryAlignment::Alignment1) => vec![
                {
                    let mut mask = [0u8; N];
                    for index in (0..N).step_by(4) {
                        mask[index] = 0xFF;
                    }
                    Simd::from_array(mask)
                },
                {
                    let mut mask = [0u8; N];
                    for index in (1..N).step_by(4) {
                        mask[index] = 0xFF;
                    }
                    Simd::from_array(mask)
                },
                {
                    let mut mask = [0u8; N];
                    for index in (2..N).step_by(4) {
                        mask[index] = 0xFF;
                    }
                    Simd::from_array(mask)
                },
                {
                    let mut mask = [0u8; N];
                    for index in (3..N).step_by(4) {
                        mask[index] = 0xFF;
                    }
                    Simd::from_array(mask)
                },
            ],
            (4, MemoryAlignment::Alignment2) => vec![
                {
                    let mut mask = [0u8; N];
                    for index in (0..N).step_by(4) {
                        mask[index] = 0xFF;
                        mask[index + 1] = 0xFF;
                    }
                    Simd::from_array(mask)
                },
                {
                    let mut mask = [0u8; N];
                    for index in (2..N).step_by(4) {
                        mask[index] = 0xFF;
                        mask[index + 1] = 0xFF;
                    }
                    Simd::from_array(mask)
                },
            ],
            // Data type size 8
            (8, MemoryAlignment::Alignment1) => vec![
                {
                    let mut mask = [0u8; N];
                    for index in (0..N).step_by(8) {
                        mask[index] = 0xFF;
                    }
                    Simd::from_array(mask)
                },
                {
                    let mut mask = [0u8; N];
                    for index in (1..N).step_by(8) {
                        mask[index] = 0xFF;
                    }
                    Simd::from_array(mask)
                },
                {
                    let mut mask = [0u8; N];
                    for index in (2..N).step_by(8) {
                        mask[index] = 0xFF;
                    }
                    Simd::from_array(mask)
                },
                {
                    let mut mask = [0u8; N];
                    for index in (3..N).step_by(8) {
                        mask[index] = 0xFF;
                    }
                    Simd::from_array(mask)
                },
                {
                    let mut mask = [0u8; N];
                    for index in (4..N).step_by(8) {
                        mask[index] = 0xFF;
                    }
                    Simd::from_array(mask)
                },
                {
                    let mut mask = [0u8; N];
                    for index in (5..N).step_by(8) {
                        mask[index] = 0xFF;
                    }
                    Simd::from_array(mask)
                },
                {
                    let mut mask = [0u8; N];
                    for index in (6..N).step_by(8) {
                        mask[index] = 0xFF;
                    }
                    Simd::from_array(mask)
                },
                {
                    let mut mask = [0u8; N];
                    for index in (7..N).step_by(8) {
                        mask[index] = 0xFF;
                    }
                    Simd::from_array(mask)
                },
            ],
            (8, MemoryAlignment::Alignment2) => vec![
                {
                    let mut mask = [0u8; N];
                    for index in (0..N).step_by(8) {
                        mask[index] = 0xFF;
                        mask[index + 1] = 0xFF;
                    }
                    Simd::from_array(mask)
                },
                {
                    let mut mask = [0u8; N];
                    for index in (2..N).step_by(8) {
                        mask[index] = 0xFF;
                        mask[index + 1] = 0xFF;
                    }
                    Simd::from_array(mask)
                },
                {
                    let mut mask = [0u8; N];
                    for index in (4..N).step_by(8) {
                        mask[index] = 0xFF;
                        mask[index + 1] = 0xFF;
                    }
                    Simd::from_array(mask)
                },
                {
                    let mut mask = [0u8; N];
                    for index in (6..N).step_by(8) {
                        mask[index] = 0xFF;
                        mask[index + 1] = 0xFF;
                    }
                    Simd::from_array(mask)
                },
            ],
            (8, MemoryAlignment::Alignment4) => vec![
                {
                    let mut mask = [0u8; N];
                    for index in (0..N).step_by(8) {
                        mask[index] = 0xFF;
                        mask[index + 1] = 0xFF;
                        mask[index + 2] = 0xFF;
                        mask[index + 3] = 0xFF;
                    }
                    Simd::from_array(mask)
                },
                {
                    let mut mask = [0u8; N];
                    for index in (4..N).step_by(8) {
                        mask[index] = 0xFF;
                        mask[index + 1] = 0xFF;
                        mask[index + 2] = 0xFF;
                        mask[index + 3] = 0xFF;
                    }
                    Simd::from_array(mask)
                },
            ],
            _ => panic!("Unexpected size/alignment combination.")
        }
    }
}

/// This algorithm is mostly the same as scanner_vector_aligned. The primary difference is that instead of doing one vector comparison,
/// multiple scans must be done per vector to scan for mis-aligned values. This adds 2 to 8 additional vector scans, based on the alignment
/// and data type. Each of these sub-scans is masked against a stagger mask to create the scan result. For example, scanning for 4-byte integer
/// with a value of 0 with an alignment of 2-bytes against <0, 0, 0, 0, 55, 0, 0, 0, 0, 0> would need to return <255, 0, 0, 0, 255, 255, ..>.
/// This is accomplished by performing a full vector scan, then masking it against the appropriate stagger mask to extract the relevant scan
/// results for that iteration. These sub-scans are OR'd together to get a run-length encoded vector of all scan matches.
impl<T: SimdType + Send + Sync, const N: usize> Scanner for ScannerVectorStaggered<T, N>
where
    LaneCount<N>: SupportedLaneCount,
{
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

        let staggered_masks = Self::get_staggered_mask(data_type_size, memory_alignment);
        let mut results = Vec::new();

        // Loop through each staggered mask and perform the scan
        for mask in staggered_masks {
            let sub_results = encoder.encode(
                snapshot_region.get_current_values_pointer(&snapshot_region_filter),
                snapshot_region.get_previous_values_pointer(&snapshot_region_filter),
                scan_parameters,
                scan_filter_parameters,
                snapshot_region_filter.get_base_address(),
                snapshot_region_filter.get_element_count(memory_alignment, data_type_size),
                &vector_comparer,
                mask,
            );
            results.extend(sub_results);
        }

        results
    }
}
