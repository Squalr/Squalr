use crate::scanners::snapshot_scanner::Scanner;
use crate::scanners::structures::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_api::structures::data_types::generics::vector_comparer::VectorComparer;
use squalr_engine_api::structures::data_types::generics::vector_generics::VectorGenerics;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use squalr_engine_api::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
use squalr_engine_api::structures::scanning::parameters::mapped::mapped_scan_parameters::MappedScanParameters;
use std::simd::cmp::SimdPartialEq;
use std::simd::{LaneCount, Simd, SupportedLaneCount};

pub struct ScannerVectorOverlapping<const N: usize>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>, {}

impl<const N: usize> ScannerVectorOverlapping<N>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
{
    fn encode_results(
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
        } else if compare_result.simd_eq(false_mask).all() {
            run_length_encoder.finalize_current_encode_with_minimum_size(N as u64, data_type_size);
        // Otherwise, there is a mix of true/false results that need to be processed manually.
        } else {
            Self::encode_remainder_results(&compare_result, run_length_encoder, data_type_size, N as u64);
        }
    }

    fn encode_remainder_results(
        compare_result: &Simd<u8, N>,
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        data_type_size: u64,
        remainder_bytes: u64,
    ) {
        let start_byte_index = (N as u64).saturating_sub(remainder_bytes);

        for byte_index in start_byte_index..(N as u64) {
            if compare_result[byte_index as usize] != 0 {
                run_length_encoder.encode_range(1);
            } else {
                run_length_encoder.finalize_current_encode_with_minimum_size(1, data_type_size);
            }
        }
    }
}

/// Implements a CPU-bound SIMD memory region scanner that is optmized for scanning for an overlapping sequence of N bytes.
/// In other words, this scan efficiently handles searching for values where the data type size is larger than the memory alignment.
impl<const N: usize> Scanner for ScannerVectorOverlapping<N>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
{
    fn get_scanner_name(&self) -> &'static str {
        &"Vector Overlapping"
    }

    /// Performs a sequential iteration over a region of memory, performing the scan comparison.
    /// A run-length encoding algorithm is used to generate new sub-regions as the scan progresses.
    fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_parameters: &MappedScanParameters,
    ) -> Vec<SnapshotRegionFilter> {
        let current_values_pointer = snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter);
        let previous_values_pointer = snapshot_region.get_previous_values_filter_pointer(&snapshot_region_filter);
        let base_address = snapshot_region_filter.get_base_address();
        let region_size = snapshot_region_filter.get_region_size();

        let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
        let data_type = scan_parameters.get_data_type();
        let data_type_size = data_type.get_size_in_bytes();
        let memory_alignment = scan_parameters.get_memory_alignment();
        let memory_alignment_size = memory_alignment as u64;
        let vector_size_in_bytes = N as u64;
        let vectorizable_iterations = region_size.saturating_sub(data_type_size.saturating_sub(1)) / vector_size_in_bytes;
        let remainder_bytes = region_size % vector_size_in_bytes;
        let remainder_ptr_offset = (vectorizable_iterations.saturating_sub(1) * vector_size_in_bytes).saturating_sub(data_type_size.saturating_sub(1)) as usize;
        let false_mask = Simd::<u8, N>::splat(0x00);
        let true_mask = Simd::<u8, N>::splat(0xFF);

        debug_assert!(data_type_size > memory_alignment_size);
        debug_assert!(memory_alignment_size == 1 || memory_alignment_size == 2 || memory_alignment_size == 4);

        // TODO: For this scan we need to ensure that each encode <actually> only encodes 1 byte, not a full match length
        // We probably <actually> want a staggered true mask? Or idk think about this.
        // But essentially we want the encode to be at an ELEMENT level, not a byte level (to allow us to right-shift elements out of bounds).

        match scan_parameters.get_compare_type() {
            ScanCompareType::Immediate(scan_compare_type_immediate) => {
                if let Some(compare_func) = data_type.get_vector_compare_func_immediate(&scan_compare_type_immediate, scan_parameters) {
                    // Compare as many full vectors as we can.
                    for index in 0..vectorizable_iterations {
                        let current_values_pointer = unsafe { current_values_pointer.add((index * vector_size_in_bytes) as usize) };
                        let mut compare_result = compare_func(current_values_pointer);

                        for overlap_index in (memory_alignment_size..data_type_size).step_by(memory_alignment_size as usize) {
                            let current_values_pointer = unsafe { current_values_pointer.add(overlap_index as usize) };
                            compare_result |= VectorGenerics::rotate_right_with_discard_max_8::<N>(compare_func(current_values_pointer), overlap_index);
                        }

                        Self::encode_results(&compare_result, &mut run_length_encoder, data_type_size, true_mask, false_mask);
                    }

                    // Handle remainder elements.
                    if remainder_bytes > 0 {
                        let current_values_pointer = unsafe { current_values_pointer.add((remainder_ptr_offset) as usize) };
                        let mut compare_result = compare_func(current_values_pointer);

                        for overlap_index in (memory_alignment_size..data_type_size).step_by(memory_alignment_size as usize) {
                            let current_values_pointer = unsafe { current_values_pointer.add(overlap_index as usize) };
                            compare_result |= VectorGenerics::rotate_right_with_discard_max_8::<N>(compare_func(current_values_pointer), overlap_index);
                        }

                        Self::encode_remainder_results(&compare_result, &mut run_length_encoder, data_type_size, remainder_bytes);
                    }
                }
            }
            ScanCompareType::Relative(scan_compare_type_relative) => {
                if let Some(compare_func) = data_type.get_vector_compare_func_relative(&scan_compare_type_relative, scan_parameters) {
                    // Compare as many full vectors as we can.
                    for index in 0..vectorizable_iterations {
                        let current_values_pointer = unsafe { current_values_pointer.add((index * vector_size_in_bytes) as usize) };
                        let previous_values_pointer = unsafe { previous_values_pointer.add((index * vector_size_in_bytes) as usize) };
                        let mut compare_result = compare_func(current_values_pointer, previous_values_pointer);

                        for overlap_index in (memory_alignment_size..data_type_size).step_by(memory_alignment_size as usize) {
                            let current_values_pointer = unsafe { current_values_pointer.add(overlap_index as usize) };
                            let previous_values_pointer = unsafe { previous_values_pointer.add(overlap_index as usize) };
                            compare_result |= VectorGenerics::rotate_right_with_discard_max_8::<N>(
                                compare_func(current_values_pointer, previous_values_pointer),
                                overlap_index,
                            );
                        }

                        Self::encode_results(&compare_result, &mut run_length_encoder, data_type_size, true_mask, false_mask);
                    }

                    // Handle remainder elements.
                    if remainder_bytes > 0 {
                        let current_values_pointer = unsafe { current_values_pointer.add((remainder_ptr_offset) as usize) };
                        let previous_values_pointer = unsafe { previous_values_pointer.add((remainder_ptr_offset) as usize) };
                        let mut compare_result = compare_func(current_values_pointer, previous_values_pointer);

                        for overlap_index in (memory_alignment_size..data_type_size).step_by(memory_alignment_size as usize) {
                            let current_values_pointer = unsafe { current_values_pointer.add(overlap_index as usize) };
                            let previous_values_pointer = unsafe { previous_values_pointer.add(overlap_index as usize) };
                            compare_result |= VectorGenerics::rotate_right_with_discard_max_8::<N>(
                                compare_func(current_values_pointer, previous_values_pointer),
                                overlap_index,
                            );
                        }

                        Self::encode_remainder_results(&compare_result, &mut run_length_encoder, data_type_size, remainder_bytes);
                    }
                }
            }
            ScanCompareType::Delta(scan_compare_type_delta) => {
                if let Some(compare_func) = data_type.get_vector_compare_func_delta(&scan_compare_type_delta, scan_parameters) {
                    // Compare as many full vectors as we can.
                    for index in 0..vectorizable_iterations {
                        let current_values_pointer = unsafe { current_values_pointer.add((index * vector_size_in_bytes) as usize) };
                        let previous_values_pointer = unsafe { previous_values_pointer.add((index * vector_size_in_bytes) as usize) };
                        let mut compare_result = compare_func(current_values_pointer, previous_values_pointer);

                        for overlap_index in (memory_alignment_size..data_type_size).step_by(memory_alignment_size as usize) {
                            let current_values_pointer = unsafe { current_values_pointer.add(overlap_index as usize) };
                            let previous_values_pointer = unsafe { previous_values_pointer.add(overlap_index as usize) };
                            compare_result |= VectorGenerics::rotate_right_with_discard_max_8::<N>(
                                compare_func(current_values_pointer, previous_values_pointer),
                                overlap_index,
                            );
                        }

                        Self::encode_results(&compare_result, &mut run_length_encoder, data_type_size, true_mask, false_mask);
                    }

                    // Handle remainder elements.
                    if remainder_bytes > 0 {
                        let current_values_pointer = unsafe { current_values_pointer.add((remainder_ptr_offset) as usize) };
                        let previous_values_pointer = unsafe { previous_values_pointer.add((remainder_ptr_offset) as usize) };
                        let mut compare_result = compare_func(current_values_pointer, previous_values_pointer);

                        for overlap_index in (memory_alignment_size..data_type_size).step_by(memory_alignment_size as usize) {
                            let current_values_pointer = unsafe { current_values_pointer.add(overlap_index as usize) };
                            let previous_values_pointer = unsafe { previous_values_pointer.add(overlap_index as usize) };
                            compare_result |= VectorGenerics::rotate_right_with_discard_max_8::<N>(
                                compare_func(current_values_pointer, previous_values_pointer),
                                overlap_index,
                            );
                        }

                        Self::encode_remainder_results(&compare_result, &mut run_length_encoder, data_type_size, remainder_bytes);
                    }
                }
            }
        };

        run_length_encoder.finalize_current_encode_with_minimum_size(0, data_type_size);
        run_length_encoder.take_result_regions()
    }
}
