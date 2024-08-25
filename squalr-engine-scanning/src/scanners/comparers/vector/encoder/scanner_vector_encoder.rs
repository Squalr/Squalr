use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::comparers::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use crate::scanners::comparers::vector::encoder::scanner_vector_comparer::ScannerVectorComparer;
use crate::scanners::comparers::vector::types::simd_type::SimdType;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use std::marker::PhantomData;
use std::simd::{LaneCount, Simd, SimdElement, SupportedLaneCount};

pub struct ScannerVectorEncoder<T, const N: usize>
where
    T: SimdElement + SimdType,
    LaneCount<N>: SupportedLaneCount,
{
    _marker: PhantomData<T>,
}

impl<T, const N: usize> ScannerVectorEncoder<T, N>
where
    T: SimdElement + SimdType,
    LaneCount<N>: SupportedLaneCount,
{
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn encode(
        &self,
        current_value_pointer: *const u8,
        previous_value_pointer: *const u8,
        scan_parameters: &ScanParameters,
        scan_filter_parameters: &ScanFilterParameters,
        base_address: u64,
        element_count: u64,
        vector_comparer: &ScannerVectorComparer<T, N>,
        true_mask: Simd<u8, N>,
    ) -> Vec<SnapshotRegionFilter> {
        let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
        let data_type = scan_filter_parameters.get_data_type();
        let data_type_size = data_type.get_size_in_bytes();
        let comparisons_per_vector = N * std::mem::size_of::<T>() / data_type_size as usize;
        let iterations = element_count / comparisons_per_vector as u64;
        let remainder_elements = element_count % comparisons_per_vector as u64;
        let false_mask = Simd::<u8, N>::splat(0);

        unsafe {
            if scan_parameters.is_immediate_comparison() {
                let immediate_value = scan_parameters.deanonymize_type(&data_type).as_ptr();
                let compare_func = vector_comparer.get_immediate_compare_func(scan_parameters.get_compare_type(), data_type);

                // Compare as many full vectors as we can
                for index in 0..iterations {
                    let current_value_pointer = current_value_pointer.add(index as usize * N);
                    let compare_result = compare_func(current_value_pointer, immediate_value);

                    self.encode_results(
                        &compare_result,
                        &mut run_length_encoder,
                        data_type_size,
                        true_mask,
                        false_mask
                    );
                }

                // Handle remainder elements
                if remainder_elements > 0 {
                    let current_value_pointer = current_value_pointer.add((iterations as usize * N) - N);
                    let compare_result = compare_func(current_value_pointer, immediate_value);

                    self.encode_remainder_results(
                        &compare_result,
                        &mut run_length_encoder,
                        data_type_size,
                        remainder_elements,
                    );
                }

            } else if scan_parameters.is_relative_comparison() {
                let compare_func = vector_comparer.get_relative_compare_func(scan_parameters.get_compare_type(), data_type);

                // Compare as many full vectors as we can
                for index in 0..iterations {
                    let current_value_pointer = current_value_pointer.add(index as usize * N);
                    let previous_value_pointer = previous_value_pointer.add(index as usize * N);
                    let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                    self.encode_results(
                        &compare_result,
                        &mut run_length_encoder,
                        data_type_size,
                        true_mask,
                        false_mask
                    );
                }

                // Handle remainder elements
                if remainder_elements > 0 {
                    let current_value_pointer = current_value_pointer.add((iterations as usize * N) - N);
                    let previous_value_pointer = previous_value_pointer.add((iterations as usize * N) - N);
                    let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                    self.encode_remainder_results(
                        &compare_result,
                        &mut run_length_encoder,
                        data_type_size,
                        remainder_elements,
                    );
                }
            } else if scan_parameters.is_relative_delta_comparison() {
                let compare_func = vector_comparer.get_relative_delta_compare_func(scan_parameters.get_compare_type(), data_type);
                let delta_arg = scan_parameters.deanonymize_type(&data_type).as_ptr();

                // Compare as many full vectors as we can
                for index in 0..iterations {
                    let current_value_pointer = current_value_pointer.add(index as usize * N);
                    let previous_value_pointer = previous_value_pointer.add(index as usize * N);
                    let compare_result = compare_func(current_value_pointer, previous_value_pointer, delta_arg);

                    self.encode_results(
                        &compare_result,
                        &mut run_length_encoder,
                        data_type_size,
                        true_mask,
                        false_mask
                    );
                }

                // Handle remainder elements
                if remainder_elements > 0 {
                    let current_value_pointer = current_value_pointer.add((iterations as usize * N) - N);
                    let compare_result = compare_func(current_value_pointer, previous_value_pointer, delta_arg);

                    self.encode_remainder_results(
                        &compare_result,
                        &mut run_length_encoder,
                        data_type_size,
                        remainder_elements,
                    );
                }
            } else {
                panic!("Unrecognized comparison");
            }
        }

        run_length_encoder.finalize_current_encode(0);

        run_length_encoder.result_regions
    }

    #[inline(always)]
    fn encode_results(
        &self,
        compare_result: &Simd<u8, N>,
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        data_type_size: u64,
        true_mask: Simd<u8, N>,
        false_mask: Simd<u8, N>,
    ) {
        // Optimization: Check if all scan results are true. This helps substantially when scanning for common values like 0.
        if compare_result.eq(&true_mask) {
            run_length_encoder.encode_range(N as u64);
        // Optimization: Check if all scan results are false. This is also a very common result, and speeds up scans.
        } else if compare_result.ne(&false_mask) {
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

    #[inline(always)]
    fn encode_remainder_results(
        &self,
        compare_result: &Simd<u8, N>,
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        data_type_size: u64,
        remainder_elements: u64,
    ) {
        let start_byte_index = (N - remainder_elements as usize * data_type_size as usize) as usize;

        for byte_index in (start_byte_index..N).step_by(data_type_size as usize) {
            if compare_result[byte_index] != 0 {
                run_length_encoder.encode_range(data_type_size);
            } else {
                run_length_encoder.finalize_current_encode(data_type_size);
            }
        }
    }
}