use squalr_engine_api::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::snapshot_scanner::Scanner;
use crate::scanners::structures::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use squalr_engine_api::structures::scanning::parameters::mapped_scan_parameters::ScanParametersCommonVector;
use squalr_engine_api::structures::{data_types::generics::vector_comparer::VectorComparer, memory::memory_alignment::MemoryAlignment};
use std::simd::cmp::SimdPartialEq;
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

    fn encode_results(
        compare_result: &Simd<u8, N>,
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        memory_alignment: u64,
        true_mask: Simd<u8, N>,
        false_mask: Simd<u8, N>,
    ) {
        // Optimization: Check if all scan results are true. This helps substantially when scanning for common values like 0.
        if compare_result.simd_eq(true_mask).all() {
            run_length_encoder.encode_range(N as u64);
        // Optimization: Check if all scan results are false. This is also a very common result, and speeds up scans.
        } else if compare_result.simd_eq(false_mask).all() {
            run_length_encoder.finalize_current_encode(N as u64);
        // Otherwise, there is a mix of true/false results that need to be processed manually.
        } else {
            Self::encode_remainder_results(compare_result, run_length_encoder, memory_alignment, N as u64);
        }
    }

    fn encode_remainder_results(
        compare_result: &Simd<u8, N>,
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        memory_alignment: u64,
        remainder_bytes: u64,
    ) {
        let start_byte_index = N.saturating_sub(remainder_bytes as usize);

        for byte_index in (start_byte_index..N).step_by(memory_alignment as usize) {
            if compare_result[byte_index] != 0 {
                run_length_encoder.encode_range(memory_alignment);
            } else {
                run_length_encoder.finalize_current_encode(memory_alignment);
            }
        }
    }
}

impl<const N: usize> Scanner<ScanParametersCommonVector> for ScannerVectorSparse<N>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
{
    fn scan_region(
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_parameters: &ScanParametersCommonVector,
    ) -> Vec<SnapshotRegionFilter> {
        let current_value_pointer = snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter);
        let previous_value_pointer = snapshot_region.get_previous_values_filter_pointer(&snapshot_region_filter);
        let base_address = snapshot_region_filter.get_base_address();
        let region_size = snapshot_region_filter.get_region_size();

        let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
        let data_type = scan_parameters.get_data_type();
        let memory_alignment = scan_parameters.get_memory_alignment();
        let memory_alignment_size = memory_alignment as u64;
        let vector_size_in_bytes = N;
        let iterations = region_size / vector_size_in_bytes as u64;
        let remainder_bytes = region_size % vector_size_in_bytes as u64;
        let remainder_ptr_offset = iterations.saturating_sub(1) as usize * vector_size_in_bytes;
        let false_mask = Simd::<u8, N>::splat(0x00);
        let true_mask = Self::get_sparse_mask(memory_alignment);
        let common_params = scan_parameters.get_common_params();

        match scan_parameters.get_compare_type() {
            ScanCompareType::Immediate(scan_compare_type_immediate) => {
                if let Some(compare_func) = data_type.get_vector_compare_func_immediate(&scan_compare_type_immediate, common_params) {
                    // Compare as many full vectors as we can.
                    for index in 0..iterations {
                        let current_value_pointer = unsafe { current_value_pointer.add(index as usize * vector_size_in_bytes) };
                        let compare_result = compare_func(current_value_pointer);

                        Self::encode_results(&compare_result, &mut run_length_encoder, memory_alignment_size, true_mask, false_mask);
                    }

                    // Handle remainder elements.
                    if remainder_bytes > 0 {
                        let current_value_pointer = unsafe { current_value_pointer.add(remainder_ptr_offset) };
                        let compare_result = compare_func(current_value_pointer);
                        Self::encode_remainder_results(&compare_result, &mut run_length_encoder, memory_alignment_size, remainder_bytes);
                    }
                }
            }
            ScanCompareType::Relative(scan_compare_type_relative) => {
                if let Some(compare_func) = data_type.get_vector_compare_func_relative(&scan_compare_type_relative, common_params) {
                    // Compare as many full vectors as we can.
                    for index in 0..iterations {
                        let current_value_pointer = unsafe { current_value_pointer.add(index as usize * vector_size_in_bytes) };
                        let previous_value_pointer = unsafe { previous_value_pointer.add(index as usize * vector_size_in_bytes) };
                        let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                        Self::encode_results(&compare_result, &mut run_length_encoder, memory_alignment_size, true_mask, false_mask);
                    }

                    // Handle remainder elements.
                    if remainder_bytes > 0 {
                        let current_value_pointer = unsafe { current_value_pointer.add(remainder_ptr_offset) };
                        let previous_value_pointer = unsafe { previous_value_pointer.add(remainder_ptr_offset) };
                        let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                        Self::encode_remainder_results(&compare_result, &mut run_length_encoder, memory_alignment_size, remainder_bytes);
                    }
                }
            }
            ScanCompareType::Delta(scan_compare_type_delta) => {
                if let Some(compare_func) = data_type.get_vector_compare_func_delta(&scan_compare_type_delta, common_params) {
                    // Compare as many full vectors as we can.
                    for index in 0..iterations {
                        let current_value_pointer = unsafe { current_value_pointer.add(index as usize * vector_size_in_bytes) };
                        let previous_value_pointer = unsafe { previous_value_pointer.add(index as usize * vector_size_in_bytes) };
                        let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                        Self::encode_results(&compare_result, &mut run_length_encoder, memory_alignment_size, true_mask, false_mask);
                    }

                    // Handle remainder elements.
                    if remainder_bytes > 0 {
                        let current_value_pointer = unsafe { current_value_pointer.add(remainder_ptr_offset) };
                        let previous_value_pointer = unsafe { previous_value_pointer.add(remainder_ptr_offset) };
                        let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                        Self::encode_remainder_results(&compare_result, &mut run_length_encoder, memory_alignment_size, remainder_bytes);
                    }
                }
            }
        }

        run_length_encoder.finalize_current_encode(0);
        run_length_encoder.take_result_regions()
    }
}
