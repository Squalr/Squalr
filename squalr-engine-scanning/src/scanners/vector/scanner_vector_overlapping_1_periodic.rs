use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::snapshot_scanner::Scanner;
use crate::scanners::structures::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_api::structures::scanning::parameters::scan_parameters::ScanParameters;
use squalr_engine_api::structures::{data_types::generics::vector_comparer::VectorComparer, scanning::comparisons::scan_compare_type::ScanCompareType};
use std::simd::cmp::SimdPartialEq;
use std::simd::{LaneCount, Simd, SupportedLaneCount};

pub struct ScannerVectorOverlapping1Periodic<const N: usize>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>, {}

impl<const N: usize> ScannerVectorOverlapping1Periodic<N>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
{
    fn encode_results(
        &self,
        compare_result: &Simd<u8, N>,
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        original_data_type_size: u64,
        true_mask: Simd<u8, N>,
        false_mask: Simd<u8, N>,
    ) {
        // Optimization: Check if all scan results are true. This helps substantially when scanning for common values like 0.
        if compare_result.simd_eq(true_mask).all() {
            run_length_encoder.encode_range(N as u64);
        // Optimization: Check if all scan results are false. This is also a very common result, and speeds up scans.
        } else if compare_result.simd_ne(false_mask).all() {
            run_length_encoder.finalize_current_encode_with_minimum_size(N as u64, original_data_type_size);
        // Otherwise, there is a mix of true/false results that need to be processed manually.
        } else {
            self.encode_remainder_results(&compare_result, run_length_encoder, original_data_type_size, N as u64);
        }
    }

    fn encode_remainder_results(
        &self,
        compare_result: &Simd<u8, N>,
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        original_data_type_size: u64,
        remainder_bytes: u64,
    ) {
        let start_byte_index = N.saturating_sub(remainder_bytes as usize);

        for byte_index in start_byte_index..N {
            if compare_result[byte_index] != 0 {
                run_length_encoder.encode_range(1);
            } else {
                run_length_encoder.finalize_current_encode_with_minimum_size(1, original_data_type_size);
            }
        }
    }
}

/// Implements a memory region scanner that is optmized/specialized for a repeated immediate value of the same byte.
impl<const N: usize> Scanner for ScannerVectorOverlapping1Periodic<N>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
{
    fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_parameters: &ScanParameters,
    ) -> Vec<SnapshotRegionFilter> {
        let current_value_pointer = snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter);
        let previous_value_pointer = snapshot_region.get_previous_values_filter_pointer(&snapshot_region_filter);
        let base_address = snapshot_region_filter.get_base_address();
        let region_size = snapshot_region_filter.get_region_size();

        let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
        let optimized_data_type = scan_parameters.get_optimized_data_type();
        let original_data_type = scan_parameters.get_original_data_type();
        let original_data_type_size = original_data_type.get_size_in_bytes();
        let vector_size_in_bytes = N;
        let iterations = region_size / vector_size_in_bytes as u64;
        let remainder_bytes = region_size % vector_size_in_bytes as u64;
        let remainder_ptr_offset = iterations.saturating_sub(1) as usize * vector_size_in_bytes;
        let false_mask = Simd::<u8, N>::splat(0x00);
        let true_mask = Simd::<u8, N>::splat(0xFF);

        unsafe {
            match scan_parameters.get_compare_type() {
                ScanCompareType::Immediate(scan_compare_type_immediate) => {
                    if let Some(compare_func) = optimized_data_type.get_vector_compare_func_immediate(&scan_compare_type_immediate, scan_parameters) {
                        // Compare as many full vectors as we can.
                        for index in 0..iterations {
                            let current_value_pointer = current_value_pointer.add(index as usize * vector_size_in_bytes);
                            let compare_result = compare_func(current_value_pointer);

                            self.encode_results(&compare_result, &mut run_length_encoder, original_data_type_size, true_mask, false_mask);
                        }

                        // Handle remainder elements.
                        if remainder_bytes > 0 {
                            let current_value_pointer = current_value_pointer.add(remainder_ptr_offset);
                            let compare_result = compare_func(current_value_pointer);

                            self.encode_remainder_results(&compare_result, &mut run_length_encoder, original_data_type_size, remainder_bytes);
                        }
                    }
                }
                ScanCompareType::Relative(scan_compare_type_relative) => {
                    if let Some(compare_func) = optimized_data_type.get_vector_compare_func_relative(&scan_compare_type_relative, scan_parameters) {
                        // Compare as many full vectors as we can.
                        for index in 0..iterations {
                            let current_value_pointer = current_value_pointer.add(index as usize * vector_size_in_bytes);
                            let previous_value_pointer = previous_value_pointer.add(index as usize * vector_size_in_bytes);
                            let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                            self.encode_results(&compare_result, &mut run_length_encoder, original_data_type_size, true_mask, false_mask);
                        }

                        // Handle remainder elements.
                        if remainder_bytes > 0 {
                            let current_value_pointer = current_value_pointer.add(remainder_ptr_offset);
                            let previous_value_pointer = previous_value_pointer.add(remainder_ptr_offset);
                            let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                            self.encode_remainder_results(&compare_result, &mut run_length_encoder, original_data_type_size, remainder_bytes);
                        }
                    }
                }
                ScanCompareType::Delta(scan_compare_type_delta) => {
                    if let Some(compare_func) = optimized_data_type.get_vector_compare_func_delta(&scan_compare_type_delta, scan_parameters) {
                        // Compare as many full vectors as we can.
                        for index in 0..iterations {
                            let current_value_pointer = current_value_pointer.add(index as usize * vector_size_in_bytes);
                            let previous_value_pointer = previous_value_pointer.add(index as usize * vector_size_in_bytes);
                            let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                            self.encode_results(&compare_result, &mut run_length_encoder, original_data_type_size, true_mask, false_mask);
                        }

                        // Handle remainder elements.
                        if remainder_bytes > 0 {
                            let current_value_pointer = current_value_pointer.add(remainder_ptr_offset);
                            let previous_value_pointer = previous_value_pointer.add(remainder_ptr_offset);
                            let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                            self.encode_remainder_results(&compare_result, &mut run_length_encoder, original_data_type_size, remainder_bytes);
                        }
                    }
                }
            }
        }

        run_length_encoder.finalize_current_encode_with_minimum_size(0, original_data_type_size);
        run_length_encoder.take_result_regions()
    }
}
