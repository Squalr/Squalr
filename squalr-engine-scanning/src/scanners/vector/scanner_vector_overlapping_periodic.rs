use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::snapshot_scanner::Scanner;
use crate::scanners::structures::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_api::structures::data_values::anonymous_value::AnonymousValue;
use squalr_engine_api::structures::data_values::data_value::DataValue;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use squalr_engine_api::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use squalr_engine_api::structures::scanning::scan_parameters_local::ScanParametersLocal;
use squalr_engine_api::structures::{data_types::generics::vector_comparer::VectorComparer, scanning::comparisons::scan_compare_type::ScanCompareType};
use std::simd::cmp::SimdPartialEq;
use std::simd::{LaneCount, Simd, SupportedLaneCount};

pub struct ScannerVectorOverlappingPeriodic<const N: usize>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>, {}

impl<const N: usize> ScannerVectorOverlappingPeriodic<N>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
{
    fn combine_results(
        &self,
        compare_results: &Vec<Simd<u8, N>>,
    ) -> Simd<u8, N> {
        let mut result = compare_results[0];

        for index in 1..compare_results.len() {
            result |= compare_results[index];
        }

        result
    }

    fn encode_results(
        &self,
        immediate: &DataValue,
        compare_results: &Vec<Simd<u8, N>>,
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        data_type_size: u64,
        memory_alignment_size: u64,
        true_mask: Simd<u8, N>,
        false_mask: Simd<u8, N>,
    ) {
        let compare_result = self.combine_results(compare_results);

        // Optimization: Check if all scan results are true. This helps substantially when scanning for common values like 0.
        if compare_result.simd_eq(true_mask).all() {
            run_length_encoder.encode_range(N as u64);
        // Optimization: Check if all scan results are false. This is also a very common result, and speeds up scans.
        } else if compare_result.simd_ne(false_mask).all() {
            run_length_encoder.finalize_current_encode(N as u64);
        // Otherwise, there is a mix of true/false results that need to be processed manually.
        } else {
            self.encode_remainder_results(immediate, &compare_result, run_length_encoder, data_type_size, memory_alignment_size, N as u64);
        }
    }

    fn encode_remainder_results(
        &self,
        immediate: &DataValue,
        compare_result: &Simd<u8, N>,
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        data_type_size: u64,
        memory_alignment_size: u64,
        remainder_bytes: u64,
    ) {
        let start_byte_index = N - remainder_bytes as usize;
        let data_type_size_padding = data_type_size.saturating_sub(memory_alignment_size);

        for byte_index in start_byte_index..N {
            if compare_result[byte_index] != 0 {
                run_length_encoder.encode_range(1);
            } else {
                // run_length_encoder.finalize_current_encode_with_padding(1, data_type_size_padding);
                run_length_encoder.finalize_current_encode_with_minimum_size(1, data_type_size);
            }
        }
    }

    pub fn get_rotation_mask(
        rotation_index: usize,
        align: usize,
    ) -> Simd<u8, N> {
        let mut mask_array = [0u8; N];

        for index in (0..(N - rotation_index)).step_by(align) {
            mask_array[index + rotation_index] = 0xFF;
        }

        Simd::from_array(mask_array)
    }

    /// Calculates the length of repeating byte patterns within a given data type and value combination.
    /// If there are no repeating patterns, the periodicity will be equal to the data type size.
    /// For example, 7C 01 7C 01 has a data typze size of 4, but a periodicity of 2.
    fn calculate_periodicity(
        &self,
        immediate_value_bytes: &[u8],
        data_type_size_bytes: u64,
    ) -> u64 {
        // Assume optimal periodicity to begin with
        let mut period = 1;

        // Loop through all remaining bytes, and increase the periodicity when we encounter a byte that violates the current assumption.
        for byte_index in 1..data_type_size_bytes as usize {
            if immediate_value_bytes[byte_index] != immediate_value_bytes[byte_index % period] {
                period = byte_index + 1;
            }
        }

        period as u64
    }

    fn build_immediate_comparers(
        &self,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Vec<Box<dyn Fn(*const u8) -> Simd<u8, N>>> {
        let data_type = scan_parameters_local.get_data_type();
        let parameters = self.build_rotated_global_parameters(scan_parameters_global, scan_parameters_local);
        let mut results = Vec::with_capacity(parameters.len());

        for parameter in parameters {
            let comparer =
                if let Some(compare_func) = data_type.get_vector_compare_func_immediate(&scan_compare_type_immediate, &parameter, scan_parameters_local) {
                    compare_func
                } else {
                    return vec![];
                };

            results.push(comparer);
        }

        results
    }

    /// Rotates immediate compare values in the global parameters, producing a new set of global parameters containing the rotated values.
    fn build_rotated_global_parameters(
        &self,
        original_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Vec<ScanParametersGlobal> {
        let data_type = scan_parameters_local.get_data_type();
        let data_type_size = data_type.get_size_in_bytes();
        let memory_alignment = scan_parameters_local.get_memory_alignment_or_default() as usize;
        let num_rotations = data_type_size as usize / memory_alignment;
        let Some(immediate_value) = original_global.deanonymize_immediate(data_type) else {
            return vec![];
        };
        let original_bytes = immediate_value.get_value_bytes();

        (0..num_rotations)
            .map(|rotation_index| {
                let mut rotated_bytes = original_bytes.clone();
                rotated_bytes.rotate_left(rotation_index);

                let mut rotated_global = original_global.clone();
                rotated_global.set_compare_immediate(Some(AnonymousValue::new_bytes(rotated_bytes)));

                rotated_global
            })
            .collect()
    }
}

/// Implements a memory region scanner to find overlapping matches using "periodicity scans with run length encoding discard".
/// In plain English, this is an algorithm that is optmized/specialized for data with repeating 1-8 byte patterns.
///     For 1-periodic scans (all same byte A)
///         Just do a normal SIMD byte scan, and discard all RLEs < data type size
///     For 2-periodic scans (repeating 2 bytes A, B)
///         Create a vector of <A,B,A,B,...> and <B,A,B,A,..>
///         Do 2-byte SIMD comparisons, and OR the results together.
///         Note that the shifted pattern could result in matching up to 2 unwanted bytes at the start/end of the RLE encoding.
///         In the RLE encoder, the first/last bytes need to be manually checked to filter these. Discard RLEs < data size.
///     For 4-periodic scans (repeating 4 bytes A, B, C, D)
///         Create a vector of <A,B,C,D,A,B,C,D,...> <B,C,D,A,B,C,D,A,..> <C,D,A,B,C,D,A,B,..> <D,A,B,C,D,A,B,C,..>
///         As before, we do 4-byte SIMD comparisons. From here we can generalize the RLE trimming.
///         We can use the first byte + size of run length to determine how much we need to trim.
///     For 8-periodic, extrapolate.
///
/// It is very important to realize that even if the user is scanning for a large data type (ie 8 bytes), it can still fall into
/// 1, 2, or 4 periodic! This will give us substantial gains over immediately going for the 8-periodic implementation.
///
/// Similarly, the same is true for byte array scans! If the array of bytes can be decomposed into periodic sequences, periodicty
/// scans will results in substantial savings, given that the array fits into a hardware vector Simd<> type.
impl<const N: usize> Scanner for ScannerVectorOverlappingPeriodic<N>
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
        let current_value_pointer = snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter);
        let previous_value_pointer = snapshot_region.get_previous_values_filter_pointer(&snapshot_region_filter);
        let base_address = snapshot_region_filter.get_base_address();
        let region_size = snapshot_region_filter.get_region_size();

        let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
        let data_type = scan_parameters_local.get_data_type();
        let data_type_size = data_type.get_size_in_bytes();
        let memory_alignment_size = scan_parameters_local.get_memory_alignment_or_default() as u64;
        let vector_size_in_bytes = N;
        let iterations = region_size / vector_size_in_bytes as u64;
        let remainder_bytes = region_size % vector_size_in_bytes as u64;
        let remainder_ptr_offset = iterations.saturating_sub(1) as usize * vector_size_in_bytes;
        let false_mask = Simd::<u8, N>::splat(0x00);
        let true_mask = Simd::<u8, N>::splat(0xFF);

        /*
        unsafe {
            match scan_parameters_global.get_compare_type() {
                ScanCompareType::Immediate(scan_compare_type_immediate) => {
                    let periodicity = if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(data_type) {
                        self.calculate_periodicity(immediate_value.get_value_bytes(), data_type_size)
                    } else {
                        0
                    };
                    let compare_funcs = self.build_immediate_comparers(&scan_compare_type_immediate, scan_parameters_global, scan_parameters_local);
                    let num_encoders = periodicity / memory_alignment_size;
                    let mut compare_results = vec![false_mask; compare_funcs.len()];

                    if compare_funcs.len() <= 0 {
                        return vec![];
                    }

                    // Compare as many full vectors as we can.
                    for index in 0..iterations {
                        let current_value_pointer = current_value_pointer.add(index as usize * vector_size_in_bytes);

                        for index in 0..compare_funcs.len() {
                            compare_results[index] = compare_funcs[index](current_value_pointer);
                        }

                        self.encode_results(
                            &compare_results,
                            &mut run_length_encoder,
                            data_type_size,
                            memory_alignment_size,
                            true_mask,
                            false_mask,
                        );
                    }

                    // Handle remainder elements.
                    if remainder_bytes > 0 {
                        let current_value_pointer = current_value_pointer.add(remainder_ptr_offset);

                        for index in 0..compare_funcs.len() {
                            compare_results[index] = compare_funcs[index](current_value_pointer);
                        }

                        let compare_result = self.interleave_results(&compare_results);
                        self.encode_remainder_results(
                            &compare_result,
                            // &compare_results,
                            &mut run_length_encoder,
                            data_type_size,
                            memory_alignment_size,
                            remainder_bytes,
                        );
                    }
                }
                ScanCompareType::Relative(scan_compare_type_relative) => {
                    if let Some(compare_func) =
                        data_type.get_vector_compare_func_relative(&scan_compare_type_relative, scan_parameters_global, scan_parameters_local)
                    {
                        // Compare as many full vectors as we can.
                        for index in 0..iterations {
                            let current_value_pointer = current_value_pointer.add(index as usize * vector_size_in_bytes);
                            let previous_value_pointer = previous_value_pointer.add(index as usize * vector_size_in_bytes);
                            let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                            // self.encode_results(&compare_result, &mut run_length_encoder, memory_alignment_size, true_mask, false_mask);
                        }

                        // Handle remainder elements.
                        if remainder_bytes > 0 {
                            let current_value_pointer = current_value_pointer.add(remainder_ptr_offset);
                            let previous_value_pointer = previous_value_pointer.add(remainder_ptr_offset);
                            let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                            // self.encode_remainder_results(&compare_result, &mut run_length_encoder, memory_alignment_size, remainder_bytes);
                        }
                    }
                }
                ScanCompareType::Delta(scan_compare_type_delta) => {
                    if let Some(compare_func) = data_type.get_vector_compare_func_delta(&scan_compare_type_delta, scan_parameters_global, scan_parameters_local)
                    {
                        // Compare as many full vectors as we can.
                        for index in 0..iterations {
                            let current_value_pointer = current_value_pointer.add(index as usize * vector_size_in_bytes);
                            let previous_value_pointer = previous_value_pointer.add(index as usize * vector_size_in_bytes);
                            let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                            // self.encode_results(&compare_result, &mut run_length_encoder, memory_alignment_size, true_mask, false_mask);
                        }

                        // Handle remainder elements.
                        if remainder_bytes > 0 {
                            let current_value_pointer = current_value_pointer.add(remainder_ptr_offset);
                            let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                            // self.encode_remainder_results(&compare_result, &mut run_length_encoder, memory_alignment_size, remainder_bytes);
                        }
                    }
                }
            }
        }*/

        run_length_encoder.finalize_current_encode(0);
        run_length_encoder.take_result_regions()
    }
}
