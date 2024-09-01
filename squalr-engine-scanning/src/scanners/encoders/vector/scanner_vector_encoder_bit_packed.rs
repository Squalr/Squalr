use crate::results::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::comparers::vector::scanner_vector_comparer::VectorComparer;
use crate::scanners::encoders::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use crate::scanners::encoders::vector::simd_type::SimdType;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use std::marker::PhantomData;
use std::simd::prelude::SimdPartialEq;
use std::simd::{LaneCount, Simd, SimdElement, SupportedLaneCount};

/// Experimental feature. From testing, this appears to offer no performance gains over byte packing.
pub struct ScannerVectorEncoderBitPacked<T, const N: usize>
where
    T: SimdElement + SimdType + PartialEq,
    LaneCount<N>: SupportedLaneCount,
    Simd<T, N>: SimdPartialEq,
{
    _marker: PhantomData<T>,
}

impl<T, const N: usize> ScannerVectorEncoderBitPacked<T, N>
where
    T: SimdElement + SimdType + PartialEq,
    LaneCount<N>: SupportedLaneCount,
    LaneCount<{ N / 2 }>: SupportedLaneCount,
    LaneCount<{ N / 4 }>: SupportedLaneCount,
    LaneCount<{ N / 8 }>: SupportedLaneCount,
    Simd<T, N>: SimdPartialEq,
{
    pub fn new() -> Self {
        Self { _marker: PhantomData }
    }

    pub fn encode(
        &self,
        current_value_pointer: *const u8,
        previous_value_pointer: *const u8,
        scan_parameters: &ScanParameters,
        scan_filter_parameters: &ScanFilterParameters,
        base_address: u64,
        region_size: u64,
        vector_comparer: &impl VectorComparer<T, N>,
        true_mask: Simd<u8, N>,
    ) -> Vec<SnapshotRegionFilter> {
        let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
        let data_type = scan_filter_parameters.get_data_type();
        let data_type_size_bytes = data_type.get_size_in_bytes();
        let vector_size_in_bytes = N;

        // The total number of vectors we can fill entirely.
        let total_iterations = region_size / vector_size_in_bytes as u64;

        // The total number of iterations where we can pack multiple vectors together.
        let bit_packed_iterations = total_iterations as u64 / data_type_size_bytes / 8;

        // The total number of iterations where we can pack multiple vectors together (this is derived from dividing and multiplying out the data type to get remainder).
        let packed_iterations = (total_iterations - bit_packed_iterations * data_type_size_bytes * 8) / data_type_size_bytes;

        // The total number of vectors remaining after packed iteration (this is derived from dividing and multiplying out the data type to get remainder).
        let unpacked_iterations = total_iterations - packed_iterations * data_type_size_bytes;

        // Now there is even more remainder that simply will not fit into vectors at all
        let remainder_bytes = region_size % vector_size_in_bytes as u64;
        let remainder_ptr_offset = total_iterations.saturating_sub(1) as usize * vector_size_in_bytes;
        let false_mask = Simd::<u8, N>::splat(0);

        unsafe {
            if scan_parameters.is_immediate_comparison() {
                let immediate_value = scan_parameters.deanonymize_type(&data_type).as_ptr();
                let compare_func = vector_comparer.get_immediate_compare_func(scan_parameters.get_compare_type(), data_type);

                match data_type_size_bytes {
                    4 => {
                        let bit_packing_size = 8 * vector_size_in_bytes * data_type_size_bytes as usize;

                        // Compare as many bit packed vectors as we can.
                        for bit_packed_iteration_index in 0..bit_packed_iterations {
                            let pointer_offset_base = bit_packing_size * bit_packed_iteration_index as usize;
                            let mut compare_results: [[Simd<u8, N>; 4]; 8] = [[Default::default(); 4]; 8];

                            // Perform 32 SIMD comparisons and store the results (4 byte data size * 8 bits to prepare for packing).
                            // Results in 8 groups of 4 bytes, which can be processed first for byte packing.
                            for bit_index in 0..8 {
                                for byte_index in 0..4 {
                                    let current_value_pointer = current_value_pointer.add(pointer_offset_base + vector_size_in_bytes * bit_index * byte_index);
                                    compare_results[bit_index][byte_index] = compare_func(current_value_pointer, immediate_value);
                                }
                            }

                            // Byte packing into 8 groups, each created from 4 interleaved SIMD vectors.
                            let mut compare_results_byte_packed: [Simd<u8, N>; 8] = [Default::default(); 8];

                            // Interleave pack the 32 comparisons down to 8 vectors (packing_group_index).
                            for packing_group_index in 0..8 {
                                let compare_results = compare_results[packing_group_index];
                                let mut compare_results_packed: Simd<u8, N> = compare_results[0];

                                for index in (0..N).step_by(data_type_size_bytes as usize) {
                                    // compare_results_packed[index] = compare_results[0][index];
                                    compare_results_packed[index + 1] = compare_results[1][index];
                                    compare_results_packed[index + 2] = compare_results[2][index];
                                    compare_results_packed[index + 3] = compare_results[3][index];
                                }

                                compare_results_byte_packed[packing_group_index] = compare_results_packed;
                            }

                            let mut compare_results_bit_packed: Simd<u8, N> = Default::default();

                            // Reduce the 8 packed vectors into 1 single vector by masking each of the bytes to extract a single bit.
                            // It doesn't really matter how we pack this.
                            for vector_byte_index in 0..N {
                                let packed_byte = (compare_results_byte_packed[0][vector_byte_index] & 0b0000_0001)
                                    | (compare_results_byte_packed[1][vector_byte_index] & 0b0000_0010)
                                    | (compare_results_byte_packed[2][vector_byte_index] & 0b0000_0100)
                                    | (compare_results_byte_packed[3][vector_byte_index] & 0b0000_1000)
                                    | (compare_results_byte_packed[4][vector_byte_index] & 0b0001_0000)
                                    | (compare_results_byte_packed[5][vector_byte_index] & 0b0010_0000)
                                    | (compare_results_byte_packed[6][vector_byte_index] & 0b0100_0000)
                                    | (compare_results_byte_packed[7][vector_byte_index] & 0b1000_0000);

                                compare_results_bit_packed[vector_byte_index] = packed_byte;
                            }

                            self.encode_results_bit_packed_4(
                                &compare_results_bit_packed,
                                &compare_results_byte_packed,
                                &compare_results,
                                &mut run_length_encoder,
                                data_type_size_bytes,
                                true_mask,
                                false_mask,
                            );
                        }

                        let byte_packed_start_address = bit_packed_iterations as usize * bit_packing_size;
                        let packing_size = vector_size_in_bytes * data_type_size_bytes as usize;

                        // Compare as many byte packed vectors as we can
                        for packed_iteration_index in 0..packed_iterations {
                            let byte_packed_pointer_offset = byte_packed_start_address + packed_iteration_index as usize * packing_size;
                            let current_value_pointers = [
                                current_value_pointer.add(byte_packed_pointer_offset + vector_size_in_bytes * 0),
                                current_value_pointer.add(byte_packed_pointer_offset + vector_size_in_bytes * 1),
                                current_value_pointer.add(byte_packed_pointer_offset + vector_size_in_bytes * 2),
                                current_value_pointer.add(byte_packed_pointer_offset + vector_size_in_bytes * 3),
                            ];

                            let compare_results = [
                                compare_func(current_value_pointers[0], immediate_value),
                                compare_func(current_value_pointers[1], immediate_value),
                                compare_func(current_value_pointers[2], immediate_value),
                                compare_func(current_value_pointers[3], immediate_value),
                            ];

                            let mut compare_results_packed: Simd<u8, N> = compare_results[0];

                            for index in (0..N).step_by(data_type_size_bytes as usize) {
                                // compare_results_packed[index] = compare_results[0][index];
                                compare_results_packed[index + 1] = compare_results[1][index];
                                compare_results_packed[index + 2] = compare_results[2][index];
                                compare_results_packed[index + 3] = compare_results[3][index];
                            }

                            self.encode_results_packed_4(
                                &compare_results_packed,
                                &compare_results,
                                &mut run_length_encoder,
                                data_type_size_bytes,
                                true_mask,
                                false_mask,
                            );
                        }

                        let unpacked_start_address = packed_iterations as usize * packing_size;

                        for unpacked_iteration_index in 0..unpacked_iterations {
                            let unpacked_pointer_offset = unpacked_start_address + unpacked_iteration_index as usize * vector_size_in_bytes;
                            let current_value_pointer = current_value_pointer.add(unpacked_pointer_offset);
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
                    _ => panic!("not implemented yet."),
                }
            } else if scan_parameters.is_relative_comparison() {
                let compare_func = vector_comparer.get_relative_compare_func(scan_parameters.get_compare_type(), data_type);

                // Compare as many full vectors as we can
                for index in 0..total_iterations {
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
            } else if scan_parameters.is_relative_delta_comparison() {
                let compare_func = vector_comparer.get_relative_delta_compare_func(scan_parameters.get_compare_type(), data_type);
                let delta_arg = scan_parameters.deanonymize_type(&data_type).as_ptr();

                // Compare as many full vectors as we can
                for index in 0..total_iterations {
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
            } else {
                panic!("Unrecognized comparison");
            }
        }

        run_length_encoder.finalize_current_encode(0);

        return run_length_encoder.take_result_regions();
    }

    fn encode_results_bit_packed_4(
        &self,
        compare_results_bit_packed: &Simd<u8, N>,
        compare_results_packed: &[Simd<u8, N>; 8],
        compare_results_unpacked: &[[Simd<u8, N>; 4]; 8],
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        data_type_size: u64,
        true_mask: Simd<u8, N>,
        false_mask: Simd<u8, N>,
    ) {
        // Optimization: Check if all scan results are true. This helps substantially when scanning for common values like 0.
        if compare_results_bit_packed.simd_eq(true_mask).all() {
            run_length_encoder.encode_range(8 * data_type_size * N as u64);
        // Optimization: Check if all scan results are false. This is also a very common result, and speeds up scans.
        } else if compare_results_bit_packed.simd_ne(false_mask).all() {
            run_length_encoder.finalize_current_encode(8 * data_type_size * N as u64);
        // Otherwise, there is a mix of true/false results that need to be processed manually.
        } else {
            for packing_index in 0..8 {
                self.encode_results_packed_4(
                    &compare_results_packed[packing_index],
                    &compare_results_unpacked[packing_index],
                    run_length_encoder,
                    data_type_size,
                    true_mask,
                    false_mask,
                );
            }
        }
    }

    fn encode_results_packed_4(
        &self,
        compare_results_packed: &Simd<u8, N>,
        compare_results_unpacked: &[Simd<u8, N>; 4],
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        data_type_size: u64,
        true_mask: Simd<u8, N>,
        false_mask: Simd<u8, N>,
    ) {
        // Optimization: Check if all scan results are true. This helps substantially when scanning for common values like 0.
        if compare_results_packed.simd_eq(true_mask).all() {
            run_length_encoder.encode_range(data_type_size * N as u64);
        // Optimization: Check if all scan results are false. This is also a very common result, and speeds up scans.
        } else if compare_results_packed.simd_ne(false_mask).all() {
            run_length_encoder.finalize_current_encode(data_type_size * N as u64);
        // Otherwise, there is a mix of true/false results that need to be processed manually.
        } else {
            for compare_result in compare_results_unpacked.iter() {
                self.encode_results(compare_result, run_length_encoder, data_type_size, true_mask, false_mask);
            }
        }
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
