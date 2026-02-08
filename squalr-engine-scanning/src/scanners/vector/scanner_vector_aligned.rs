use crate::scanners::snapshot_scanner::Scanner;
use crate::scanners::structures::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use squalr_engine_api::structures::data_types::generics::vector_comparer::VectorComparer;
use squalr_engine_api::structures::data_types::generics::vector_function::GetVectorFunction;
use squalr_engine_api::structures::data_types::generics::vector_generics::VectorGenerics;
use squalr_engine_api::structures::data_types::generics::vector_lane_count::VectorLaneCount;
use squalr_engine_api::structures::scanning::comparisons::scan_function_vector::ScanFunctionVector;
use squalr_engine_api::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
use squalr_engine_api::structures::scanning::plans::element_scan::snapshot_filter_element_scan_plan::SnapshotFilterElementScanPlan;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use std::simd::Simd;
use std::simd::cmp::SimdPartialEq;

pub struct ScannerVectorAligned<const N: usize>
where
    VectorLaneCount<N>: VectorComparer<N> + GetVectorFunction<N>, {}

impl<const N: usize> ScannerVectorAligned<N>
where
    VectorLaneCount<N>: VectorComparer<N> + GetVectorFunction<N>,
{
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

/// Implements a CPU-bound SIMD memory region scanner that is optmized for scanning for a sequence of N bytes.
/// In other words, this scan efficiently handles searching for values where the data type size is exactly equal to the memory alignment.
impl<const N: usize> Scanner for ScannerVectorAligned<N>
where
    VectorLaneCount<N>: VectorComparer<N> + GetVectorFunction<N>,
{
    fn get_scanner_name(&self) -> &'static str {
        &"Vector (Aligned)"
    }

    /// Performs a sequential iteration over a region of memory, performing the scan comparison.
    /// A run-length encoding algorithm is used to generate new sub-regions as the scan progresses.
    fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Vec<SnapshotRegionFilter> {
        let current_values_pointer = snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter);
        let previous_value_pointer = snapshot_region.get_previous_values_filter_pointer(&snapshot_region_filter);
        let base_address = snapshot_region_filter.get_base_address();
        let region_size = snapshot_region_filter.get_region_size();

        let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
        let data_type_size = snapshot_filter_element_scan_plan.get_unit_size_in_bytes();
        let memory_alignment_size = snapshot_filter_element_scan_plan.get_memory_alignment() as u64;

        let vectorization_plan = VectorGenerics::plan_vector_scan::<N>(region_size, data_type_size, memory_alignment_size);
        let vectorizable_iterations = vectorization_plan.get_vectorizable_iterations();
        let remainder_ptr_offset = vectorization_plan.get_remainder_ptr_offset();
        let remainder_bytes = vectorization_plan.get_remainder_bytes();

        let false_mask = Simd::<u8, N>::splat(0x00);
        let true_mask = Simd::<u8, N>::splat(0xFF);

        debug_assert!(vectorizable_iterations > 0);
        debug_assert!(data_type_size == memory_alignment_size);
        debug_assert!(memory_alignment_size == 1 || memory_alignment_size == 2 || memory_alignment_size == 4 || memory_alignment_size == 8);

        if let Some(vector_compare_func) = snapshot_filter_element_scan_plan.get_scan_function_vector() {
            match vector_compare_func {
                ScanFunctionVector::Immediate(compare_func) => {
                    // Compare as many full vectors as we can.
                    for index in 0..vectorizable_iterations {
                        let current_values_pointer = unsafe { current_values_pointer.add((index * vectorization_plan.vector_size_in_bytes) as usize) };
                        let compare_result = compare_func(current_values_pointer);

                        Self::encode_results(&compare_result, &mut run_length_encoder, memory_alignment_size, true_mask, false_mask);
                    }

                    // Handle remainder elements.
                    if remainder_bytes > 0 {
                        let current_values_pointer = unsafe { current_values_pointer.add(remainder_ptr_offset as usize) };
                        let compare_result = compare_func(current_values_pointer);

                        Self::encode_remainder_results(&compare_result, &mut run_length_encoder, memory_alignment_size, remainder_bytes);
                    }
                }
                ScanFunctionVector::RelativeOrDelta(compare_func) => {
                    // Compare as many full vectors as we can.
                    for index in 0..vectorizable_iterations {
                        let current_values_pointer = unsafe { current_values_pointer.add((index * vectorization_plan.vector_size_in_bytes) as usize) };
                        let previous_value_pointer = unsafe { previous_value_pointer.add((index * vectorization_plan.vector_size_in_bytes) as usize) };
                        let compare_result = compare_func(current_values_pointer, previous_value_pointer);

                        Self::encode_results(&compare_result, &mut run_length_encoder, memory_alignment_size, true_mask, false_mask);
                    }

                    // Handle remainder elements.
                    if remainder_bytes > 0 {
                        let current_values_pointer = unsafe { current_values_pointer.add(remainder_ptr_offset as usize) };
                        let previous_value_pointer = unsafe { previous_value_pointer.add(remainder_ptr_offset as usize) };
                        let compare_result = compare_func(current_values_pointer, previous_value_pointer);

                        Self::encode_remainder_results(&compare_result, &mut run_length_encoder, memory_alignment_size, remainder_bytes);
                    }
                }
            }
        }

        run_length_encoder.finalize_current_encode(0);
        run_length_encoder.take_result_regions()
    }
}
