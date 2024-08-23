use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::comparers::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use crate::scanners::comparers::vector::encoder::scanner_vector_comparer_128::ScannerVectorComparer;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use std::simd::{u8x16, Simd};
use std::sync::Once;

pub struct ScannerVectorEncoder {
}

impl ScannerVectorEncoder {
    fn new(
    ) -> Self {
        Self { }
    }
    
    pub fn get_instance(
    ) -> &'static ScannerVectorEncoder {
        static mut INSTANCE: Option<ScannerVectorEncoder> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = ScannerVectorEncoder::new();
                INSTANCE = Some(instance);
            });

            return INSTANCE.as_ref().unwrap_unchecked();
        }
    }

    fn unpack_vec(input: u8x16) -> [u8x16; 8] {
        let mut result = [u8x16::splat(0); 8];

        for bit_index in 0..8 {
            // Create a mask to isolate the bit at position bit_index
            let mask = u8x16::splat(1 << bit_index);
            // Mask the input and shift the bit to the least significant position
            let masked_bits = (input & mask) >> bit_index;
            // Multiply by 0xFF to get either 0x00 or 0xFF
            result[bit_index as usize] = masked_bits * u8x16::splat(0xFF);
        }
    
        return result;
    }

    pub fn encode(
        &self,
        current_value_pointer: *const u8,
        previous_value_pointer: *const u8,
        scan_parameters: &ScanParameters,
        scan_filter_parameters: &ScanFilterParameters,
        base_address: u64,
        element_count: u64,
    ) -> Vec<SnapshotRegionFilter> {
        let comparer = ScannerVectorComparer::get_instance();
        let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
        let data_type = scan_filter_parameters.get_data_type();
        let data_type_size = data_type.size_in_bytes();
        let memory_alignment = scan_filter_parameters.get_memory_alignment_or_default() as u64;
        let iterations = element_count / 128;
        let true_mask = u8x16::splat(0xFF);
        let false_mask = u8x16::splat(0);
        
        unsafe {
            let mut encode_results = |compare_result : Simd<u8, 16>| {
                // Optimization: Check every scan result passed (batch size 128).
                if compare_result.eq(&true_mask) {
                    run_length_encoder.encode_range(128 * data_type_size);
                // Optimization: Check every scan result failed (batch size 128).
                } else if compare_result.eq(&false_mask) {
                    run_length_encoder.finalize_current_encode_unsized(128 * data_type_size);
                // Otherwise, it's a mix and extra effort is required.
                } else {
                    let unpacked_results = Self::unpack_vec(compare_result);
                    // Optimization: Try to unpack the bit vector into 8 byte vectors for SIMD comparisons
                    for unpacked_vector in unpacked_results {
                        // Optimization: Check every scan result passed (batch size 16).
                        if unpacked_vector.eq(&true_mask) {
                            run_length_encoder.encode_range(16 * data_type_size);
                        // Optimization: Check every scan result failed (batch size 16).
                        } else if unpacked_vector.eq(&false_mask) {
                            run_length_encoder.finalize_current_encode_unsized(16 * data_type_size);
                        }
                        else {
                            // Otherwise, it's still a mix, and now we need to manually check each byte.
                            for byte_index in 0..16 {
                                // Remap the index to the packing order of the hardware vector.
                                let byte_index = (byte_index % 8) * 2 + (byte_index / 8);
                                if unpacked_vector[byte_index] == 0xFF {
                                    run_length_encoder.encode_range(data_type_size);
                                } else {
                                    run_length_encoder.finalize_current_encode_unsized(data_type_size);
                                }
                            }
                        }
                    }
                }
            };

            if scan_parameters.is_immediate_comparison() {
                let immediate_value = scan_parameters.deanonymize_type(&data_type).as_ptr();
                let compare_func = comparer.get_immediate_compare_func(scan_parameters.get_compare_type(), data_type);

                for index in 0..iterations {
                    let current_value_pointer = current_value_pointer.add(index as usize * 128 * data_type_size as usize);
                    let compare_result = compare_func(current_value_pointer, immediate_value);

                    encode_results(compare_result);
                }
            } else if scan_parameters.is_relative_comparison() {
                let compare_func = comparer.get_relative_compare_func(scan_parameters.get_compare_type(), data_type);

                for index in 0..iterations {
                    let current_value_pointer = current_value_pointer.add(index as usize * memory_alignment as usize * 16 * data_type_size as usize);
                    let previous_value_pointer = previous_value_pointer.add(index as usize * memory_alignment as usize * 16 * data_type_size as usize);
                    let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                    encode_results(compare_result);
                }
            } else if scan_parameters.is_immediate_comparison() {
                let compare_func = comparer.get_immediate_compare_func(scan_parameters.get_compare_type(), data_type);
                let delta_arg = scan_parameters.deanonymize_type(&data_type).as_ptr();

                for index in 0..iterations {
                    let current_value_pointer = current_value_pointer.add(index as usize * memory_alignment as usize * 16 * data_type_size as usize);
                    let compare_result = compare_func(current_value_pointer, delta_arg);

                    encode_results(compare_result);
                }
            } else {
                panic!("Unrecognized comparison");
            }
        }

        run_length_encoder.finalize_current_encode_unsized(0);
        
        return run_length_encoder.result_regions;
    }
}
