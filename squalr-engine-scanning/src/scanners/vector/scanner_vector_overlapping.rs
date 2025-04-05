use squalr_engine_api::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::snapshot_scanner::Scanner;
use crate::scanners::structures::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_api::structures::data_values::anonymous_value::AnonymousValue;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use squalr_engine_api::structures::scanning::parameters::mapped_scan_parameters::ScanParametersCommonVector;
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

    fn interleave_results(
        &self,
        compare_results: &Vec<Simd<u8, N>>,
    ) -> Simd<u8, N> {
        // Fuck we're losing data arent we, we rotate out of our window -- everything that spills out gets gg'd.
        // We would need to track a global "over-shift" while encoding, which is agony.
        // This problem is incredibly ass.
        // Our scan is great at answering the question "does this group of data type size contain the start or end of a matching primitive"
        // The pain in the ass of course is when the answer to this question is split across two vectors.
        // Maybe, just maybe, there is some way to keep the prior vector around for stitching, but this adds incredible complexity.
        // The less state I have to track, the better.
        // So what even is the solution? Like sure, perhaps I can create an over-read mask basically taking the elements shifted out-of-range,
        // And put them into some new vec, and pass this around everywhere. Pretty shit.
        // Alternatively, I can potentially <not> shift anything, note the shift internally by the index of the vec
        // idk whatever.

        // Maybe the better idea is to do a reducing operation, from data-size aligned to points (ie mask every n).
        // BUT actually this needs to be more of a tournament -- collapse every n, such that the mask requires all data_sized groups to be set
        // 0xFF{x4} => 0xFF, but {0xFFx3} => nope.
        // Well, hm, perhaps no, again, our scanner is designed to pick up a very different case.
        // Our scanner does not stagger 4 vectors, it rotates the immediate.
        // This produces a <very> different answer to the question.

        // Wait is the answer really just take the grouping max?
        // Like a literal <data_size_container> <count set bytes> <select most set bytes, should never be a tie>
        // Lemme think tho

        // This is a dumb problem I seem to have invented for myself.

        // Another thought was (rightmost-bit wins), which is also sus

        // The goal here is to interleave each rotated vector, such that for each grouping of data type size,
        match compare_results.len() {
            1 => compare_results[0],
            2 => compare_results[0] | compare_results[1].rotate_elements_right::<1>(),
            4 => {
                compare_results[0]
                    | compare_results[1].rotate_elements_right::<1>()
                    | compare_results[2].rotate_elements_right::<2>()
                    | compare_results[3].rotate_elements_right::<3>()
            }
            8 => {
                compare_results[0]
                    | compare_results[1].rotate_elements_right::<1>()
                    | compare_results[2].rotate_elements_right::<2>()
                    | compare_results[3].rotate_elements_right::<3>()
                    | compare_results[4].rotate_elements_right::<4>()
                    | compare_results[5].rotate_elements_right::<5>()
                    | compare_results[6].rotate_elements_right::<6>()
                    | compare_results[7].rotate_elements_right::<7>()
            }
            _ => {
                // Should never happen, only primitive sizes are supported.
                debug_assert!(false);
                Simd::splat(0x00)
            }
        }
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

    fn encode_results(
        &self,
        compare_results: &Vec<Simd<u8, N>>,
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        data_type_size: u64,
        memory_alignment_size: u64,
        true_mask: Simd<u8, N>,
        false_mask: Simd<u8, N>,
    ) {
        let compare_result = self.interleave_results(compare_results);

        // Optimization: Check if all scan results are true. This helps substantially when scanning for common values like 0.
        if compare_result.simd_eq(true_mask).all() {
            run_length_encoder.encode_range(N as u64);
        // Optimization: Check if all scan results are false. This is also a very common result, and speeds up scans.
        } else if compare_result.simd_eq(false_mask).all() {
            run_length_encoder.finalize_current_encode(N as u64);
        // Otherwise, there is a mix of true/false results that need to be processed manually.
        } else {
            self.encode_remainder_results(&compare_result, run_length_encoder, data_type_size, memory_alignment_size, N as u64);
        }
    }

    fn encode_remainder_results(
        &self,
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

    /*
    fn build_immediate_comparers(
        &self,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
        scan_parameters: &ScanParametersCommon,
    ) -> Vec<Box<dyn Fn(*const u8) -> Simd<u8, N>>> {
        let data_type = scan_parameters.get_data_type();
        let parameters = self.build_rotated_global_parameters(user_scan_parameters_global, scan_parameters);
        let mut results = Vec::with_capacity(parameters.len());

        for parameter in parameters {
            let comparer = if let Some(compare_func) = data_type.get_vector_compare_func_immediate(&scan_compare_type_immediate, &parameter, scan_parameters) {
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
        original_global: &UserScanParametersGlobal,
        scan_parameters: &UserScanParametersLocal,
    ) -> Vec<UserScanParametersGlobal> {
        let data_type = scan_parameters.get_data_type();
        let data_type_size = data_type.get_size_in_bytes();
        let memory_alignment = scan_parameters.get_memory_alignment_or_default() as usize;
        let num_rotations = data_type_size as usize / memory_alignment;
        let Some(compare_immediate) = original_global.get_compare_immediate() else {
            return vec![];
        };
        let Ok(immediate_value) = data_type.deanonymize_value(compare_immediate) else {
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
    } */
}

/// Overlapping scans are the single most complex case to handle due to the base addresses not being aligned.
///
/// These fall into a few distinct cases:
/// - A) Long pattern scans, falling under "string search algorithms", ie byte array scans.
/// - B) Vectorized overlapping scans of primitive values.
///     - C) Periodic scans, which is a sub-case of vectorized overlapping scans.
///
/// This implementation handles case B & C. To solve this, we take any immediates we are comparing and rotate them N times.
/// For example, if scanning for an int32 of alignment 1, we need to perform sizeof(int32) / 1 => 4 rotations, producing:
///     - 00 00 00 01, 00 00 01 00, 00 01 00 00, 01 00 00 00
///
/// Note: To handle case C, we can also cleverly shift periodic values such as 00 00 00 00 -- which does not need to be
/// shifted at all. Or the value 01 00 01 00, which only needs to be shifted into 01 00 01 00 and 00 01 00 01.
///
/// Now when we iterate and do comparisons, we compare 4 values at once, OR the results together. However, this only
/// tells us if "a data-size aligned address contains one of the rotated values" -- but it does not tell us which one.
/// That said, if the entire vector is true or false, we do not care, and can encode the entire range of results.
/// However, if the scan result has a partial match, we need to do extra work to pull out which rotated immediate matched.
impl<const N: usize> Scanner<ScanParametersCommonVector> for ScannerVectorOverlapping<N>
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
        let data_type_size = data_type.get_size_in_bytes();
        let memory_alignment_size = scan_parameters.get_memory_alignment() as u64;
        let vector_size_in_bytes = N;
        let iterations = region_size / vector_size_in_bytes as u64;
        let remainder_bytes = region_size % vector_size_in_bytes as u64;
        let remainder_ptr_offset = iterations.saturating_sub(1) as usize * vector_size_in_bytes;
        let false_mask = Simd::<u8, N>::splat(0x00);
        let true_mask = Simd::<u8, N>::splat(0xFF);

        /*
               unsafe {
                   match scan_parameters.get_compare_type() {
                       ScanCompareType::Immediate(scan_compare_type_immediate) => {
                           /*
                           let periodicity = if let Some(immediate_value) = user_scan_parameters_global.deanonymize_immediate(data_type) {
                               self.calculate_periodicity(immediate_value.get_value_bytes(), data_type_size)
                           } else {
                               0
                           }; */
                           // let compare_funcs = self.build_immediate_comparers(&scan_compare_type_immediate, scan_parameters);
                           let compare_funcs = vec![];
                           // let num_encoders = periodicity / memory_alignment_size;
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
                           if let Some(compare_func) = data_type.get_vector_compare_func_relative(&scan_compare_type_relative, scan_parameters) {
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
                           if let Some(compare_func) = data_type.get_vector_compare_func_delta(&scan_compare_type_delta, scan_parameters) {
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
                   }
               }
        */
        run_length_encoder.finalize_current_encode(0);
        run_length_encoder.take_result_regions()
    }
}
