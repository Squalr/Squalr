use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::encoders::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use squalr_engine_common::structures::data_types::data_type::DataType;
use squalr_engine_common::structures::scanning::scan_compare_type::ScanCompareType;
use std::simd::prelude::SimdPartialEq;
use std::simd::{LaneCount, Simd, SupportedLaneCount};

pub struct ScannerVectorEncoder<const N: usize>
where
    LaneCount<N>: SupportedLaneCount, {}

impl<const N: usize> ScannerVectorEncoder<N>
where
    LaneCount<N>: SupportedLaneCount,
{
    pub fn new() -> Self {
        Self {}
    }

    pub fn vector_encode(
        &self,
        current_value_pointer: *const u8,
        previous_value_pointer: *const u8,
        scan_parameters: &ScanParameters,
        data_type: &Box<dyn DataType>,
        base_address: u64,
        region_size: u64,
        true_mask: Simd<u8, N>,
    ) -> Vec<SnapshotRegionFilter> {
        let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
        let data_type_size_bytes = data_type.get_size_in_bytes();
        let vector_size_in_bytes = N;
        let iterations = region_size / vector_size_in_bytes as u64;
        let remainder_bytes = region_size % vector_size_in_bytes as u64;
        let remainder_ptr_offset = iterations.saturating_sub(1) as usize * vector_size_in_bytes;
        let false_mask = Simd::<u8, N>::splat(0);

        unsafe {
            match scan_parameters.get_compare_type() {
                ScanCompareType::Immediate(scan_compare_type_immediate) => {
                    let immediate_value = scan_parameters.deanonymize_type(&data_type).as_ptr();
                    let compare_func = data_type.get_vector_immediate_compare_func(scan_parameters.get_compare_type(), data_type);

                    // Compare as many full vectors as we can
                    for index in 0..iterations {
                        let current_value_pointer = current_value_pointer.add(index as usize * vector_size_in_bytes);
                        let compare_result = compare_func(current_value_pointer, immediate_value);

                        self.encode_results(&compare_result, &mut run_length_encoder, data_type_size_bytes, true_mask, false_mask);
                    }

                    // Handle remainder elements
                    if remainder_bytes > 0 {
                        let current_value_pointer = current_value_pointer.add(remainder_ptr_offset);
                        let compare_result = compare_func(current_value_pointer, immediate_value);
                        self.encode_remainder_results(&compare_result, &mut run_length_encoder, data_type_size_bytes, remainder_bytes);
                    }
                }
                ScanCompareType::Relative(scan_compare_type_relative) => {
                    let compare_func = data_type.get_vector_relative_compare_func(scan_parameters.get_compare_type(), data_type);

                    // Compare as many full vectors as we can
                    for index in 0..iterations {
                        let current_value_pointer = current_value_pointer.add(index as usize * vector_size_in_bytes);
                        let previous_value_pointer = previous_value_pointer.add(index as usize * vector_size_in_bytes);
                        let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                        self.encode_results(&compare_result, &mut run_length_encoder, data_type_size_bytes, true_mask, false_mask);
                    }

                    // Handle remainder elements
                    if remainder_bytes > 0 {
                        let current_value_pointer = current_value_pointer.add(remainder_ptr_offset);
                        let previous_value_pointer = previous_value_pointer.add(remainder_ptr_offset);
                        let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                        self.encode_remainder_results(&compare_result, &mut run_length_encoder, data_type_size_bytes, remainder_bytes);
                    }
                }
                ScanCompareType::Delta(scan_compare_type_delta) => {
                    let compare_func = data_type.get_vector_delta_compare_func(scan_parameters.get_compare_type(), data_type);
                    let delta_arg = scan_parameters.deanonymize_type(&data_type).as_ptr();

                    // Compare as many full vectors as we can
                    for index in 0..iterations {
                        let current_value_pointer = current_value_pointer.add(index as usize * vector_size_in_bytes);
                        let previous_value_pointer = previous_value_pointer.add(index as usize * vector_size_in_bytes);
                        let compare_result = compare_func(current_value_pointer, previous_value_pointer, delta_arg);

                        self.encode_results(&compare_result, &mut run_length_encoder, data_type_size_bytes, true_mask, false_mask);
                    }

                    // Handle remainder elements
                    if remainder_bytes > 0 {
                        let current_value_pointer = current_value_pointer.add(remainder_ptr_offset);
                        let compare_result = compare_func(current_value_pointer, previous_value_pointer, delta_arg);

                        self.encode_remainder_results(&compare_result, &mut run_length_encoder, data_type_size_bytes, remainder_bytes);
                    }
                }
            }
        }

        run_length_encoder.finalize_current_encode(0);
        run_length_encoder.take_result_regions()
    }

    fn encode_results(
        &self,
        compare_result: &Simd<u8, N>,
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        data_type_size: u64,
        true_mask: Simd<u8, N>,
        false_mask: Simd<u8, N>,
    ) {
        // Optimization: Check if all scan results are true. This helps substantially when scanning for common values like 0.
        if compare_result.simd_eq(true_mask).all() {
            run_length_encoder.encode_range(N as u64);
        // Optimization: Check if all scan results are false. This is also a very common result, and speeds up scans.
        } else if compare_result.simd_ne(false_mask).all() {
            run_length_encoder.finalize_current_encode(N as u64);
        // Otherwise, there is a mix of true/false results that need to be processed manually.
        } else {
            for byte_index in (0..N).step_by(data_type_size as usize) {
                if compare_result[byte_index] != 0 {
                    run_length_encoder.encode_range(data_type_size);
                } else {
                    run_length_encoder.finalize_current_encode(data_type_size);
                }
            }
        }
    }

    fn encode_remainder_results(
        &self,
        compare_result: &Simd<u8, N>,
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        data_type_size: u64,
        remainder_bytes: u64,
    ) {
        let start_byte_index = N - remainder_bytes as usize;

        for byte_index in (start_byte_index..N).step_by(data_type_size as usize) {
            if compare_result[byte_index] != 0 {
                run_length_encoder.encode_range(data_type_size);
            } else {
                run_length_encoder.finalize_current_encode(data_type_size);
            }
        }
    }
}
