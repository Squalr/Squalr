use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::snapshot_scanner::Scanner;
use crate::scanners::structures::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_api::structures::data_types::generics::vector_comparer::VectorComparer;
use squalr_engine_api::structures::data_types::generics::vector_generics::VectorGenerics;
use squalr_engine_api::structures::data_values::data_value::DataValue;
use squalr_engine_api::structures::scanning::parameters::scan_parameters::ScanParameters;
use std::ptr;
use std::simd::cmp::SimdPartialEq;
use std::simd::{LaneCount, Simd, SupportedLaneCount};

pub struct ScannerVectorOverlappingNPeriodic<const N: usize>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>, {}

impl<const N: usize> ScannerVectorOverlappingNPeriodic<N>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
{
    fn encode_results(
        &self,
        compare_result: &Simd<u8, N>,
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        data_type_size_padding: u64,
        true_mask: Simd<u8, N>,
        false_mask: Simd<u8, N>,
        vector_compare_size: u64,
    ) {
        // Optimization: Check if all scan results are true. This helps substantially when scanning for common values like 0.
        if compare_result.simd_eq(true_mask).all() {
            run_length_encoder.encode_range(vector_compare_size);
            // Optimization: Check if all scan results are false. This is also a very common result, and speeds up scans.
        } else if compare_result.simd_eq(false_mask).all() {
            run_length_encoder.finalize_current_encode_with_padding(vector_compare_size, data_type_size_padding);
        // Otherwise, there is a mix of true/false results that need to be processed manually.
        } else {
            self.encode_remainder_results(
                &compare_result,
                run_length_encoder,
                data_type_size_padding,
                vector_compare_size,
                vector_compare_size,
            );
        }
    }

    fn encode_remainder_results(
        &self,
        compare_result: &Simd<u8, N>,
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        data_type_size_padding: u64,
        remainder_bytes: u64,
        vector_compare_size: u64,
    ) {
        let start_byte_index = vector_compare_size.saturating_sub(remainder_bytes);

        for byte_index in start_byte_index..vector_compare_size {
            if compare_result[byte_index as usize] != 0 {
                run_length_encoder.encode_range(1);
            } else {
                run_length_encoder.finalize_current_encode_with_padding(1, data_type_size_padding);
            }
        }
    }
}

/// Implements a memory region scanner that is optmized for scanning for an overlapping sequence of N bytes.
/// For example, even scanning for something like `00 01 02 03`
impl<const N: usize> Scanner for ScannerVectorOverlappingNPeriodic<N>
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
        let base_address = snapshot_region_filter.get_base_address();
        let region_size = snapshot_region_filter.get_region_size();

        let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
        let original_data_type = scan_parameters.get_original_data_type();
        let original_data_type_size = original_data_type.get_size_in_bytes();
        let data_type_size_padding = original_data_type_size.saturating_sub(scan_parameters.get_memory_alignment_or_default() as u64);
        let vector_size_in_bytes = N;
        let vector_underflow = original_data_type_size as usize;
        let vector_compare_size = vector_size_in_bytes.saturating_sub(vector_underflow) as u64;
        let iterations = region_size / vector_compare_size;
        let remainder_bytes = region_size % vector_compare_size;
        let remainder_ptr_offset = (iterations.saturating_sub(1) * vector_compare_size) as usize;
        let false_mask = Simd::<u8, N>::splat(0x00);
        let true_mask = Simd::<u8, N>::splat(0xFF);

        let scan_immedate = match scan_parameters.get_data_value() {
            Some(scan_immediate) => scan_immediate,
            None => {
                log::error!("Failed to get compare immediate for 2-periodic scan.");
                return vec![];
            }
        };

        let load_nth_byte_vec = |scan_immedate: &DataValue, byte_index: usize| {
            let byte_vec = Simd::<u8, N>::splat(scan_immedate.get_value_bytes()[byte_index]);

            Box::new(move |current_values_ptr| {
                let current_values = unsafe { Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [u8; N])) };
                VectorGenerics::transmute_mask::<u8, N, N>(current_values.simd_eq(byte_vec))
            })
        };

        match original_data_type_size {
            2 => {
                let compare_func_byte_0 = load_nth_byte_vec(&scan_immedate, 0);
                let compare_func_byte_1 = load_nth_byte_vec(&scan_immedate, 1);

                // Compare as many full vectors as we can.
                for index in 0..iterations {
                    let current_value_pointer = unsafe { current_value_pointer.add((index * vector_compare_size) as usize) };
                    let compare_results_0 = compare_func_byte_0(current_value_pointer);
                    let compare_results_1 = compare_func_byte_1(current_value_pointer).rotate_elements_left::<1>();
                    let compare_result = compare_results_0 & compare_results_1;

                    self.encode_results(
                        &compare_result,
                        &mut run_length_encoder,
                        data_type_size_padding,
                        true_mask,
                        false_mask,
                        vector_compare_size,
                    );
                }

                // Handle remainder elements.
                if remainder_bytes > 0 {
                    let compare_results_0 = unsafe { compare_func_byte_0(current_value_pointer.add(remainder_ptr_offset)) };
                    let compare_results_1 = unsafe { compare_func_byte_1(current_value_pointer.add(remainder_ptr_offset)).rotate_elements_left::<1>() };
                    let compare_result = compare_results_0 & compare_results_1;

                    self.encode_remainder_results(
                        &compare_result,
                        &mut run_length_encoder,
                        data_type_size_padding,
                        remainder_bytes,
                        vector_compare_size,
                    );
                }
            }
            4 => {
                let compare_func_byte_0 = load_nth_byte_vec(&scan_immedate, 0);
                let compare_func_byte_1 = load_nth_byte_vec(&scan_immedate, 1);
                let compare_func_byte_2 = load_nth_byte_vec(&scan_immedate, 2);
                let compare_func_byte_3 = load_nth_byte_vec(&scan_immedate, 3);

                // Compare as many full vectors as we can.
                for index in 0..iterations {
                    let current_value_pointer = unsafe { current_value_pointer.add((index * vector_compare_size) as usize) };
                    let compare_results_0 = compare_func_byte_0(current_value_pointer);
                    let compare_results_1 = compare_func_byte_1(current_value_pointer).rotate_elements_left::<1>();
                    let compare_results_2 = compare_func_byte_2(current_value_pointer).rotate_elements_left::<2>();
                    let compare_results_3 = compare_func_byte_3(current_value_pointer).rotate_elements_left::<3>();
                    let compare_result = compare_results_0 & compare_results_1 & compare_results_2 & compare_results_3;

                    self.encode_results(
                        &compare_result,
                        &mut run_length_encoder,
                        data_type_size_padding,
                        true_mask,
                        false_mask,
                        vector_compare_size,
                    );
                }

                // Handle remainder elements.
                if remainder_bytes > 0 {
                    let remainder_value_pointer = unsafe { current_value_pointer.add(remainder_ptr_offset) };
                    let compare_results_0 = compare_func_byte_0(remainder_value_pointer);
                    let compare_results_1 = compare_func_byte_1(remainder_value_pointer).rotate_elements_left::<1>();
                    let compare_results_2 = compare_func_byte_2(remainder_value_pointer).rotate_elements_left::<2>();
                    let compare_results_3 = compare_func_byte_3(remainder_value_pointer).rotate_elements_left::<3>();
                    let compare_result = compare_results_0 & compare_results_1 & compare_results_2 & compare_results_3;

                    self.encode_remainder_results(
                        &compare_result,
                        &mut run_length_encoder,
                        data_type_size_padding,
                        remainder_bytes,
                        vector_compare_size,
                    );
                }
            }
            8 => {
                let compare_func_byte_0 = load_nth_byte_vec(&scan_immedate, 0);
                let compare_func_byte_1 = load_nth_byte_vec(&scan_immedate, 1);
                let compare_func_byte_2 = load_nth_byte_vec(&scan_immedate, 2);
                let compare_func_byte_3 = load_nth_byte_vec(&scan_immedate, 3);
                let compare_func_byte_4 = load_nth_byte_vec(&scan_immedate, 4);
                let compare_func_byte_5 = load_nth_byte_vec(&scan_immedate, 5);
                let compare_func_byte_6 = load_nth_byte_vec(&scan_immedate, 6);
                let compare_func_byte_7 = load_nth_byte_vec(&scan_immedate, 7);

                // Compare as many full vectors as we can.
                for index in 0..iterations {
                    let current_value_pointer = unsafe { current_value_pointer.add((index * vector_compare_size) as usize) };
                    let compare_results_0 = compare_func_byte_0(current_value_pointer);
                    let compare_results_1 = compare_func_byte_1(current_value_pointer).rotate_elements_left::<1>();
                    let compare_results_2 = compare_func_byte_2(current_value_pointer).rotate_elements_left::<2>();
                    let compare_results_3 = compare_func_byte_3(current_value_pointer).rotate_elements_left::<3>();
                    let compare_results_4 = compare_func_byte_4(current_value_pointer).rotate_elements_left::<4>();
                    let compare_results_5 = compare_func_byte_5(current_value_pointer).rotate_elements_left::<5>();
                    let compare_results_6 = compare_func_byte_6(current_value_pointer).rotate_elements_left::<6>();
                    let compare_results_7 = compare_func_byte_7(current_value_pointer).rotate_elements_left::<7>();
                    let compare_result = compare_results_0
                        & compare_results_1
                        & compare_results_2
                        & compare_results_3
                        & compare_results_4
                        & compare_results_5
                        & compare_results_6
                        & compare_results_7;

                    self.encode_results(
                        &compare_result,
                        &mut run_length_encoder,
                        data_type_size_padding,
                        true_mask,
                        false_mask,
                        vector_compare_size,
                    );
                }

                // Handle remainder elements.
                if remainder_bytes > 0 {
                    let remainder_value_pointer = unsafe { current_value_pointer.add(remainder_ptr_offset) };
                    let compare_results_0 = compare_func_byte_0(remainder_value_pointer);
                    let compare_results_1 = compare_func_byte_1(remainder_value_pointer).rotate_elements_left::<1>();
                    let compare_results_2 = compare_func_byte_2(remainder_value_pointer).rotate_elements_left::<2>();
                    let compare_results_3 = compare_func_byte_3(remainder_value_pointer).rotate_elements_left::<3>();
                    let compare_results_4 = compare_func_byte_4(remainder_value_pointer).rotate_elements_left::<4>();
                    let compare_results_5 = compare_func_byte_5(remainder_value_pointer).rotate_elements_left::<5>();
                    let compare_results_6 = compare_func_byte_6(remainder_value_pointer).rotate_elements_left::<6>();
                    let compare_results_7 = compare_func_byte_7(remainder_value_pointer).rotate_elements_left::<7>();
                    let compare_result = compare_results_0
                        & compare_results_1
                        & compare_results_2
                        & compare_results_3
                        & compare_results_4
                        & compare_results_5
                        & compare_results_6
                        & compare_results_7;

                    self.encode_remainder_results(
                        &compare_result,
                        &mut run_length_encoder,
                        data_type_size_padding,
                        remainder_bytes,
                        vector_compare_size,
                    );
                }
            }
            _ => {
                log::error!("Unsupported data type size provided to 2-periodic scan!");
                return vec![];
            }
        }

        run_length_encoder.finalize_current_encode_with_padding(0, data_type_size_padding);
        run_length_encoder.take_result_regions()
    }
}
