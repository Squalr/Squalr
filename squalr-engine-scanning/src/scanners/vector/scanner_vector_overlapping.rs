use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::snapshot_scanner::Scanner;
use crate::scanners::structures::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_api::structures::data_values::anonymous_value::AnonymousValue;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use squalr_engine_api::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use squalr_engine_api::structures::scanning::scan_parameters_local::ScanParametersLocal;
use squalr_engine_api::structures::{data_types::generics::vector_comparer::VectorComparer, scanning::comparisons::scan_compare_type::ScanCompareType};
use std::simd::cmp::SimdPartialEq;
use std::simd::{LaneCount, Simd, SupportedLaneCount};

pub struct ScannerVectorOverlapping<const N: usize>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>, {}

impl<const N: usize> ScannerVectorOverlapping<N>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
{
    fn combine_results(
        &self,
        compare_results: &Vec<Simd<u8, N>>,
    ) -> Simd<u8, N> {
        let mut compare_result = compare_results[0];

        for index in 1..compare_results.len() {
            compare_result |= compare_results[index];
        }

        compare_result
    }

    fn encode_results(
        &self,
        compare_results: &Vec<Simd<u8, N>>,
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        data_type_size: u64,
        memory_alignment: u64,
        true_mask: Simd<u8, N>,
        false_mask: Simd<u8, N>,
    ) {
        let compare_result = self.combine_results(compare_results);

        // Optimization: Check if all scan results are true. This helps substantially when scanning for common values like 0.
        // JIRA: If we're using periodic overlapping scans, this will almost never be a valid check, and a fake optimization.
        if compare_result.simd_eq(true_mask).all() {
            run_length_encoder.encode_range(N as u64);
        // Optimization: Check if all scan results are false. This is also a very common result, and speeds up scans.
        } else if compare_result.simd_ne(false_mask).all() {
            run_length_encoder.finalize_current_encode(N as u64);
        // Otherwise, there is a mix of true/false results that need to be processed manually.
        } else {
            self.encode_remainder_results(compare_result, compare_results, run_length_encoder, data_type_size, memory_alignment, N as u64);
        }
    }

    fn encode_remainder_results(
        &self,
        compare_result: Simd<u8, N>,
        compare_results: &Vec<Simd<u8, N>>,
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        data_type_size: u64,
        memory_alignment: u64,
        remainder_bytes: u64,
    ) {
        let data_type_size_padding = data_type_size.saturating_sub(memory_alignment);
        let mut byte_index = N - remainder_bytes as usize;
        let mut current_rotation_index = 0;

        debug_assert!((byte_index as u64) % memory_alignment == 0);

        while byte_index < N {
            // Optimization: If the combined results for this index is set to 0, there are no matches, and we can skip a full alignment length.
            if compare_result[byte_index] == 0 {
                run_length_encoder.finalize_current_encode_with_padding(memory_alignment, data_type_size_padding);
                byte_index += memory_alignment as usize;
                continue;
            }

            let mut rotation_index = 0;
            let mut rotation_shift = 0;

            // Otherwise, we need to figure out which rotated compare result matched.
            for compare_index in 0..compare_results.len() {
                rotation_index = (current_rotation_index + compare_index) % compare_results.len();
                if compare_results[rotation_index][byte_index] != 0 {
                    break;
                }
                rotation_shift += 1;
            }

            // Should not be possible to rotate back to the same spot, a match should be guaranteed.
            debug_assert!(rotation_shift < compare_results.len());

            current_rotation_index = rotation_index;

            // Check if our rotational mode has changed, and skip any of those matches.
            if rotation_shift > 0 {
                run_length_encoder.finalize_current_encode_with_padding(rotation_shift as u64, data_type_size_padding);
                byte_index += rotation_shift + rotation_shift;
            } else {
                let remainder_elements = memory_alignment.saturating_sub(rotation_shift as u64);

                // There should always be one or more valid match.
                debug_assert!(remainder_elements > 0);

                run_length_encoder.encode_range(remainder_elements);
                byte_index += remainder_elements as usize;
            }
        }
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

/// Overlapping scans are the single most complex case to handle due to the base addresses not being aligned.
///
/// These fall into a few distinct cases:
/// - A) Periodic scans.
/// - B) Long patterns, falling under "string search algorithms", ie byte array scans.
/// - C) Vectorized overlapping.
///
/// This implementation handles case C. To solve this, we take any immediates we are comparing and rotate them N times.
/// For example, if scanning for an int32 of alignment 1, we need to perform sizeof(int32) / 1 => 4 rotations, producing:
///     - 00 00 00 01, 00 00 01 00, 00 01 00 00, 01 00 00 00
/// Now when we iterate and do comparisons, we compare 4 values at once, OR the results together. However, this only
/// tells us if "a data-size aligned address contains one of the rotated values" -- but it does not tell us which one.
/// That said, if the entire vector is true or false, we do not care, and can encode the entire range of results.
/// However, if the scan result has a partial match, we need to do extra work to pull out which rotated immediate matched.
///
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
        let current_value_pointer = snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter);
        let previous_value_pointer = snapshot_region.get_previous_values_filter_pointer(&snapshot_region_filter);
        let base_address = snapshot_region_filter.get_base_address();
        let region_size = snapshot_region_filter.get_region_size();

        let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
        let data_type = scan_parameters_local.get_data_type();
        let data_type_size = data_type.get_size_in_bytes();
        let memory_alignment = scan_parameters_local.get_memory_alignment_or_default() as u64;
        let vector_size_in_bytes = N;
        let iterations = region_size / vector_size_in_bytes as u64;
        let remainder_bytes = region_size % vector_size_in_bytes as u64;
        let remainder_ptr_offset = iterations.saturating_sub(1) as usize * vector_size_in_bytes;
        let false_mask = Simd::<u8, N>::splat(0x00);
        let true_mask = Simd::<u8, N>::splat(0xFF);

        unsafe {
            match scan_parameters_global.get_compare_type() {
                ScanCompareType::Immediate(scan_compare_type_immediate) => {
                    let compare_funcs = self.build_immediate_comparers(&scan_compare_type_immediate, scan_parameters_global, scan_parameters_local);
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
                            memory_alignment,
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

                        let compare_result = self.combine_results(&compare_results);
                        self.encode_remainder_results(
                            compare_result,
                            &compare_results,
                            &mut run_length_encoder,
                            data_type_size,
                            memory_alignment,
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

                            // self.encode_results(&compare_result, &mut run_length_encoder, memory_alignment, true_mask, false_mask);
                        }

                        // Handle remainder elements.
                        if remainder_bytes > 0 {
                            let current_value_pointer = current_value_pointer.add(remainder_ptr_offset);
                            let previous_value_pointer = previous_value_pointer.add(remainder_ptr_offset);
                            let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                            // self.encode_remainder_results(&compare_result, &mut run_length_encoder, memory_alignment, remainder_bytes);
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

                            // self.encode_results(&compare_result, &mut run_length_encoder, memory_alignment, true_mask, false_mask);
                        }

                        // Handle remainder elements.
                        if remainder_bytes > 0 {
                            let current_value_pointer = current_value_pointer.add(remainder_ptr_offset);
                            let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                            // self.encode_remainder_results(&compare_result, &mut run_length_encoder, memory_alignment, remainder_bytes);
                        }
                    }
                }
            }
        }

        run_length_encoder.finalize_current_encode(0);
        run_length_encoder.take_result_regions()
    }
}
