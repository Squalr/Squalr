use crate::scanners::snapshot_scanner::Scanner;
use crate::scanners::structures::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_api::structures::data_types::generics::vector_comparer::VectorComparer;
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
            Self::encode_remainder_results(
                &compare_result,
                run_length_encoder,
                data_type_size_padding,
                vector_compare_size,
                vector_compare_size,
            );
        }
    }

    fn encode_remainder_results(
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
impl<const N: usize> Scanner for ScannerVectorOverlapping<N>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
{
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
        let data_type_size_padding = data_type_size.saturating_sub(scan_parameters.get_memory_alignment() as u64);
        let vector_size_in_bytes = N;
        let vector_overflow = data_type_size as usize;
        let vector_compare_size = vector_size_in_bytes.saturating_add(vector_overflow) as u64;
        let iterations = region_size / vector_compare_size;
        let remainder_bytes = region_size % vector_compare_size;
        let remainder_ptr_offset = (iterations.saturating_sub(1) * vector_compare_size) as usize;
        let false_mask = Simd::<u8, N>::splat(0x00);
        let true_mask = Simd::<u8, N>::splat(0xFF);

        match scan_parameters.get_compare_type() {
            ScanCompareType::Immediate(scan_compare_type_immediate) => {
                if let Some(compare_func) = data_type.get_vector_compare_func_immediate(&scan_compare_type_immediate, scan_parameters) {
                    match data_type_size {
                        2 => {
                            // Compare as many full vectors as we can.
                            for index in 0..iterations {
                                let current_values_pointer = unsafe { current_values_pointer.add((index * vector_compare_size) as usize) };
                                let compare_results_0 = compare_func(current_values_pointer);
                                let compare_results_1 = compare_func(unsafe { current_values_pointer.add(1) }).rotate_elements_left::<1>();
                                let compare_result = compare_results_0 & compare_results_1;

                                Self::encode_results(
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
                                let current_values_pointer = unsafe { current_values_pointer.add(remainder_ptr_offset) };
                                let compare_results_0 = compare_func(current_values_pointer);
                                let compare_results_1 = unsafe { compare_func(current_values_pointer.add(1)).rotate_elements_left::<1>() };
                                let compare_result = compare_results_0 | compare_results_1;

                                Self::encode_remainder_results(
                                    &compare_result,
                                    &mut run_length_encoder,
                                    data_type_size_padding,
                                    remainder_bytes,
                                    vector_compare_size,
                                );
                            }
                        }
                        4 => {
                            // Compare as many full vectors as we can.
                            for index in 0..iterations {
                                let current_values_pointer = unsafe { current_values_pointer.add((index * vector_compare_size) as usize) };
                                let compare_results_0 = compare_func(current_values_pointer);
                                let compare_results_1 = compare_func(unsafe { current_values_pointer.add(1) }).rotate_elements_left::<1>();
                                let compare_results_2 = compare_func(unsafe { current_values_pointer.add(2) }).rotate_elements_left::<2>();
                                let compare_results_3 = compare_func(unsafe { current_values_pointer.add(3) }).rotate_elements_left::<3>();
                                let compare_result = compare_results_0 | compare_results_1 | compare_results_2 | compare_results_3;

                                Self::encode_results(
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
                                let current_values_pointer = unsafe { current_values_pointer.add(remainder_ptr_offset) };
                                let compare_results_0 = compare_func(current_values_pointer);
                                let compare_results_1 = compare_func(unsafe { current_values_pointer.add(1) }).rotate_elements_left::<1>();
                                let compare_results_2 = compare_func(unsafe { current_values_pointer.add(2) }).rotate_elements_left::<2>();
                                let compare_results_3 = compare_func(unsafe { current_values_pointer.add(3) }).rotate_elements_left::<3>();
                                let compare_result = compare_results_0 & compare_results_1 & compare_results_2 & compare_results_3;

                                Self::encode_remainder_results(
                                    &compare_result,
                                    &mut run_length_encoder,
                                    data_type_size_padding,
                                    remainder_bytes,
                                    vector_compare_size,
                                );
                            }
                        }
                        8 => {
                            // Compare as many full vectors as we can.
                            for index in 0..iterations {
                                let current_values_pointer = unsafe { current_values_pointer.add((index * vector_compare_size) as usize) };
                                let compare_results_0 = compare_func(current_values_pointer);
                                let compare_results_1 = compare_func(unsafe { current_values_pointer.add(1) }).rotate_elements_left::<1>();
                                let compare_results_2 = compare_func(unsafe { current_values_pointer.add(2) }).rotate_elements_left::<2>();
                                let compare_results_3 = compare_func(unsafe { current_values_pointer.add(3) }).rotate_elements_left::<3>();
                                let compare_results_4 = compare_func(unsafe { current_values_pointer.add(4) }).rotate_elements_left::<4>();
                                let compare_results_5 = compare_func(unsafe { current_values_pointer.add(5) }).rotate_elements_left::<5>();
                                let compare_results_6 = compare_func(unsafe { current_values_pointer.add(6) }).rotate_elements_left::<6>();
                                let compare_results_7 = compare_func(unsafe { current_values_pointer.add(7) }).rotate_elements_left::<7>();
                                let compare_result = compare_results_0
                                    & compare_results_1
                                    & compare_results_2
                                    & compare_results_3
                                    & compare_results_4
                                    & compare_results_5
                                    & compare_results_6
                                    & compare_results_7;

                                Self::encode_results(
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
                                let current_values_pointer = unsafe { current_values_pointer.add(remainder_ptr_offset) };
                                let compare_results_0 = compare_func(current_values_pointer);
                                let compare_results_1 = compare_func(unsafe { current_values_pointer.add(1) }).rotate_elements_left::<1>();
                                let compare_results_2 = compare_func(unsafe { current_values_pointer.add(2) }).rotate_elements_left::<2>();
                                let compare_results_3 = compare_func(unsafe { current_values_pointer.add(3) }).rotate_elements_left::<3>();
                                let compare_results_4 = compare_func(unsafe { current_values_pointer.add(4) }).rotate_elements_left::<4>();
                                let compare_results_5 = compare_func(unsafe { current_values_pointer.add(5) }).rotate_elements_left::<5>();
                                let compare_results_6 = compare_func(unsafe { current_values_pointer.add(6) }).rotate_elements_left::<6>();
                                let compare_results_7 = compare_func(unsafe { current_values_pointer.add(7) }).rotate_elements_left::<7>();
                                let compare_result = compare_results_0
                                    & compare_results_1
                                    & compare_results_2
                                    & compare_results_3
                                    & compare_results_4
                                    & compare_results_5
                                    & compare_results_6
                                    & compare_results_7;

                                Self::encode_remainder_results(
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
                }
            }
            ScanCompareType::Relative(scan_compare_type_relative) => {
                if let Some(compare_func) = data_type.get_vector_compare_func_relative(&scan_compare_type_relative, scan_parameters) {
                    match data_type_size {
                        2 => {
                            // Compare as many full vectors as we can.
                            for index in 0..iterations {
                                let current_values_pointer = unsafe { current_values_pointer.add((index * vector_compare_size) as usize) };
                                let previous_values_pointer = unsafe { previous_values_pointer.add((index * vector_compare_size) as usize) };
                                let compare_results_0 = compare_func(current_values_pointer, previous_values_pointer);
                                let compare_results_1 =
                                    unsafe { compare_func(current_values_pointer.add(1), previous_values_pointer.add(1)).rotate_elements_left::<1>() };
                                let compare_result = compare_results_0 | compare_results_1;

                                Self::encode_results(
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
                                let current_values_pointer = unsafe { current_values_pointer.add(remainder_ptr_offset) };
                                let previous_values_pointer = unsafe { previous_values_pointer.add(remainder_ptr_offset) };
                                let compare_results_0 = compare_func(current_values_pointer, previous_values_pointer);
                                let compare_results_1 =
                                    unsafe { compare_func(current_values_pointer.add(1), previous_values_pointer.add(1)).rotate_elements_left::<1>() };
                                let compare_result = compare_results_0 | compare_results_1;

                                Self::encode_remainder_results(
                                    &compare_result,
                                    &mut run_length_encoder,
                                    data_type_size_padding,
                                    remainder_bytes,
                                    vector_compare_size,
                                );
                            }
                        }
                        4 => {
                            // Compare as many full vectors as we can.
                            for index in 0..iterations {
                                let current_values_pointer = unsafe { current_values_pointer.add((index * vector_compare_size) as usize) };
                                let previous_values_pointer = unsafe { previous_values_pointer.add((index * vector_compare_size) as usize) };
                                let compare_results_0 = compare_func(current_values_pointer, previous_values_pointer);
                                let compare_results_1 =
                                    unsafe { compare_func(current_values_pointer.add(1), previous_values_pointer.add(1)).rotate_elements_left::<1>() };
                                let compare_results_2 =
                                    unsafe { compare_func(current_values_pointer.add(2), previous_values_pointer.add(2)).rotate_elements_left::<2>() };
                                let compare_results_3 =
                                    unsafe { compare_func(current_values_pointer.add(3), previous_values_pointer.add(3)).rotate_elements_left::<3>() };
                                let compare_result = compare_results_0 | compare_results_1 | compare_results_2 | compare_results_3;

                                Self::encode_results(
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
                                let current_values_pointer = unsafe { current_values_pointer.add(remainder_ptr_offset) };
                                let previous_values_pointer = unsafe { previous_values_pointer.add(remainder_ptr_offset) };
                                let compare_results_0 = compare_func(current_values_pointer, previous_values_pointer);
                                let compare_results_1 =
                                    unsafe { compare_func(current_values_pointer.add(1), previous_values_pointer.add(1)).rotate_elements_left::<1>() };
                                let compare_results_2 =
                                    unsafe { compare_func(current_values_pointer.add(2), previous_values_pointer.add(2)).rotate_elements_left::<2>() };
                                let compare_results_3 =
                                    unsafe { compare_func(current_values_pointer.add(3), previous_values_pointer.add(3)).rotate_elements_left::<3>() };
                                let compare_result = compare_results_0 | compare_results_1 | compare_results_2 | compare_results_3;

                                Self::encode_remainder_results(
                                    &compare_result,
                                    &mut run_length_encoder,
                                    data_type_size_padding,
                                    remainder_bytes,
                                    vector_compare_size,
                                );
                            }
                        }
                        8 => {
                            // Compare as many full vectors as we can.
                            for index in 0..iterations {
                                let current_values_pointer = unsafe { current_values_pointer.add((index * vector_compare_size) as usize) };
                                let previous_values_pointer = unsafe { previous_values_pointer.add((index * vector_compare_size) as usize) };
                                let compare_results_0 = compare_func(current_values_pointer, previous_values_pointer);
                                let compare_results_1 =
                                    unsafe { compare_func(current_values_pointer.add(1), previous_values_pointer.add(1)).rotate_elements_left::<1>() };
                                let compare_results_2 =
                                    unsafe { compare_func(current_values_pointer.add(2), previous_values_pointer.add(2)).rotate_elements_left::<2>() };
                                let compare_results_3 =
                                    unsafe { compare_func(current_values_pointer.add(3), previous_values_pointer.add(3)).rotate_elements_left::<3>() };
                                let compare_results_4 =
                                    unsafe { compare_func(current_values_pointer.add(4), previous_values_pointer.add(4)).rotate_elements_left::<4>() };
                                let compare_results_5 =
                                    unsafe { compare_func(current_values_pointer.add(5), previous_values_pointer.add(5)).rotate_elements_left::<5>() };
                                let compare_results_6 =
                                    unsafe { compare_func(current_values_pointer.add(6), previous_values_pointer.add(6)).rotate_elements_left::<6>() };
                                let compare_results_7 =
                                    unsafe { compare_func(current_values_pointer.add(7), previous_values_pointer.add(7)).rotate_elements_left::<7>() };
                                let compare_result = compare_results_0
                                    | compare_results_1
                                    | compare_results_2
                                    | compare_results_3
                                    | compare_results_4
                                    | compare_results_5
                                    | compare_results_6
                                    | compare_results_7;

                                Self::encode_results(
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
                                let current_values_pointer = unsafe { current_values_pointer.add(remainder_ptr_offset) };
                                let previous_values_pointer = unsafe { previous_values_pointer.add(remainder_ptr_offset) };
                                let compare_results_0 = compare_func(current_values_pointer, previous_values_pointer);
                                let compare_results_1 =
                                    unsafe { compare_func(current_values_pointer.add(1), previous_values_pointer.add(1)).rotate_elements_left::<1>() };
                                let compare_results_2 =
                                    unsafe { compare_func(current_values_pointer.add(2), previous_values_pointer.add(2)).rotate_elements_left::<2>() };
                                let compare_results_3 =
                                    unsafe { compare_func(current_values_pointer.add(3), previous_values_pointer.add(3)).rotate_elements_left::<3>() };
                                let compare_results_4 =
                                    unsafe { compare_func(current_values_pointer.add(4), previous_values_pointer.add(4)).rotate_elements_left::<4>() };
                                let compare_results_5 =
                                    unsafe { compare_func(current_values_pointer.add(5), previous_values_pointer.add(5)).rotate_elements_left::<5>() };
                                let compare_results_6 =
                                    unsafe { compare_func(current_values_pointer.add(6), previous_values_pointer.add(6)).rotate_elements_left::<6>() };
                                let compare_results_7 =
                                    unsafe { compare_func(current_values_pointer.add(7), previous_values_pointer.add(7)).rotate_elements_left::<7>() };
                                let compare_result = compare_results_0
                                    | compare_results_1
                                    | compare_results_2
                                    | compare_results_3
                                    | compare_results_4
                                    | compare_results_5
                                    | compare_results_6
                                    | compare_results_7;

                                Self::encode_remainder_results(
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
                }
            }
            ScanCompareType::Delta(scan_compare_type_delta) => {
                if let Some(compare_func) = data_type.get_vector_compare_func_delta(&scan_compare_type_delta, scan_parameters) {
                    match data_type_size {
                        2 => {
                            // Compare as many full vectors as we can.
                            for index in 0..iterations {
                                let current_values_pointer = unsafe { current_values_pointer.add((index * vector_compare_size) as usize) };
                                let previous_values_pointer = unsafe { previous_values_pointer.add((index * vector_compare_size) as usize) };
                                let compare_results_0 = compare_func(current_values_pointer, previous_values_pointer);
                                let compare_results_1 =
                                    unsafe { compare_func(current_values_pointer.add(1), previous_values_pointer.add(1)).rotate_elements_left::<1>() };
                                let compare_result = compare_results_0 | compare_results_1;

                                Self::encode_results(
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
                                let current_values_pointer = unsafe { current_values_pointer.add(remainder_ptr_offset) };
                                let previous_values_pointer = unsafe { previous_values_pointer.add(remainder_ptr_offset) };
                                let compare_results_0 = compare_func(current_values_pointer, previous_values_pointer);
                                let compare_results_1 =
                                    unsafe { compare_func(current_values_pointer.add(1), previous_values_pointer.add(1)).rotate_elements_left::<1>() };
                                let compare_result = compare_results_0 | compare_results_1;

                                Self::encode_remainder_results(
                                    &compare_result,
                                    &mut run_length_encoder,
                                    data_type_size_padding,
                                    remainder_bytes,
                                    vector_compare_size,
                                );
                            }
                        }
                        4 => {
                            // Compare as many full vectors as we can.
                            for index in 0..iterations {
                                let current_values_pointer = unsafe { current_values_pointer.add((index * vector_compare_size) as usize) };
                                let previous_values_pointer = unsafe { previous_values_pointer.add((index * vector_compare_size) as usize) };
                                let compare_results_0 = compare_func(current_values_pointer, previous_values_pointer);
                                let compare_results_1 =
                                    unsafe { compare_func(current_values_pointer.add(1), previous_values_pointer.add(1)).rotate_elements_left::<1>() };
                                let compare_results_2 =
                                    unsafe { compare_func(current_values_pointer.add(2), previous_values_pointer.add(2)).rotate_elements_left::<2>() };
                                let compare_results_3 =
                                    unsafe { compare_func(current_values_pointer.add(3), previous_values_pointer.add(3)).rotate_elements_left::<3>() };
                                let compare_result = compare_results_0 | compare_results_1 | compare_results_2 | compare_results_3;

                                Self::encode_results(
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
                                let current_values_pointer = unsafe { current_values_pointer.add(remainder_ptr_offset) };
                                let previous_values_pointer = unsafe { previous_values_pointer.add(remainder_ptr_offset) };
                                let compare_results_0 = compare_func(current_values_pointer, previous_values_pointer);
                                let compare_results_1 =
                                    unsafe { compare_func(current_values_pointer.add(1), previous_values_pointer.add(1)).rotate_elements_left::<1>() };
                                let compare_results_2 =
                                    unsafe { compare_func(current_values_pointer.add(2), previous_values_pointer.add(2)).rotate_elements_left::<2>() };
                                let compare_results_3 =
                                    unsafe { compare_func(current_values_pointer.add(3), previous_values_pointer.add(3)).rotate_elements_left::<3>() };
                                let compare_result = compare_results_0 | compare_results_1 | compare_results_2 | compare_results_3;

                                Self::encode_remainder_results(
                                    &compare_result,
                                    &mut run_length_encoder,
                                    data_type_size_padding,
                                    remainder_bytes,
                                    vector_compare_size,
                                );
                            }
                        }
                        8 => {
                            // Compare as many full vectors as we can.
                            for index in 0..iterations {
                                let current_values_pointer = unsafe { current_values_pointer.add((index * vector_compare_size) as usize) };
                                let previous_values_pointer = unsafe { previous_values_pointer.add((index * vector_compare_size) as usize) };
                                let compare_results_0 = compare_func(current_values_pointer, previous_values_pointer);
                                let compare_results_1 =
                                    unsafe { compare_func(current_values_pointer.add(1), previous_values_pointer.add(1)).rotate_elements_left::<1>() };
                                let compare_results_2 =
                                    unsafe { compare_func(current_values_pointer.add(2), previous_values_pointer.add(2)).rotate_elements_left::<2>() };
                                let compare_results_3 =
                                    unsafe { compare_func(current_values_pointer.add(3), previous_values_pointer.add(3)).rotate_elements_left::<3>() };
                                let compare_results_4 =
                                    unsafe { compare_func(current_values_pointer.add(4), previous_values_pointer.add(4)).rotate_elements_left::<4>() };
                                let compare_results_5 =
                                    unsafe { compare_func(current_values_pointer.add(5), previous_values_pointer.add(5)).rotate_elements_left::<5>() };
                                let compare_results_6 =
                                    unsafe { compare_func(current_values_pointer.add(6), previous_values_pointer.add(6)).rotate_elements_left::<6>() };
                                let compare_results_7 =
                                    unsafe { compare_func(current_values_pointer.add(7), previous_values_pointer.add(7)).rotate_elements_left::<7>() };
                                let compare_result = compare_results_0
                                    | compare_results_1
                                    | compare_results_2
                                    | compare_results_3
                                    | compare_results_4
                                    | compare_results_5
                                    | compare_results_6
                                    | compare_results_7;

                                Self::encode_results(
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
                                let current_values_pointer = unsafe { current_values_pointer.add(remainder_ptr_offset) };
                                let previous_values_pointer = unsafe { previous_values_pointer.add(remainder_ptr_offset) };
                                let compare_results_0 = compare_func(current_values_pointer, previous_values_pointer);
                                let compare_results_1 =
                                    unsafe { compare_func(current_values_pointer.add(1), previous_values_pointer.add(1)).rotate_elements_left::<1>() };
                                let compare_results_2 =
                                    unsafe { compare_func(current_values_pointer.add(2), previous_values_pointer.add(2)).rotate_elements_left::<2>() };
                                let compare_results_3 =
                                    unsafe { compare_func(current_values_pointer.add(3), previous_values_pointer.add(3)).rotate_elements_left::<3>() };
                                let compare_results_4 =
                                    unsafe { compare_func(current_values_pointer.add(4), previous_values_pointer.add(4)).rotate_elements_left::<4>() };
                                let compare_results_5 =
                                    unsafe { compare_func(current_values_pointer.add(5), previous_values_pointer.add(5)).rotate_elements_left::<5>() };
                                let compare_results_6 =
                                    unsafe { compare_func(current_values_pointer.add(6), previous_values_pointer.add(6)).rotate_elements_left::<6>() };
                                let compare_results_7 =
                                    unsafe { compare_func(current_values_pointer.add(7), previous_values_pointer.add(7)).rotate_elements_left::<7>() };
                                let compare_result = compare_results_0
                                    | compare_results_1
                                    | compare_results_2
                                    | compare_results_3
                                    | compare_results_4
                                    | compare_results_5
                                    | compare_results_6
                                    | compare_results_7;

                                Self::encode_remainder_results(
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
                }
            }
        };

        run_length_encoder.finalize_current_encode_with_padding(0, data_type_size_padding);
        run_length_encoder.take_result_regions()
    }
}
