use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::comparers::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use crate::scanners::comparers::vector::encoder::scanner_vector_comparer::ScannerVectorComparer;
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

    pub fn encode(
        &self,
        current_value_pointer: *const u8,
        previous_value_pointer: *const u8,
        scan_parameters: &ScanParameters,
        scan_filter_parameters: &ScanFilterParameters,
        base_address: u64,
        element_count: u64,
    ) -> Vec<SnapshotRegionFilter> {
        let comparer = ScannerVectorComparer::<128>::get_instance();
        let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
        let data_type = scan_filter_parameters.get_data_type();
        let data_type_size = data_type.size_in_bytes();
        let memory_alignment = scan_filter_parameters.get_memory_alignment_or_default() as u64;
        let iterations = element_count / data_type_size;
        let true_mask = u8x16::splat(0xFF);
        let false_mask = u8x16::splat(0);
        
        unsafe {
            let mut encode_results = |compare_result : Simd<u8, 16>| {
                // Optimization: Check every scan result passed (batch size 16 bytes).
                if compare_result.eq(&true_mask) {
                    run_length_encoder.encode_range(16);
                // Optimization: Check every scan result failed (batch size 16 bytes).
                } else if compare_result.eq(&false_mask) {
                    run_length_encoder.finalize_current_encode_unsized(16);
                // Otherwise, it's a mix and extra effort is required.
                } else {
                    // Otherwise, it's still a mix, and now we need to manually check each byte.
                    for byte_index in (0..16).step_by(data_type_size as usize) {
                        if compare_result[byte_index] != 0 {
                            run_length_encoder.encode_range(data_type_size);
                        } else {
                            run_length_encoder.finalize_current_encode_unsized(data_type_size);
                        }
                    }
                }
            };

            if scan_parameters.is_immediate_comparison() {
                let immediate_value = scan_parameters.deanonymize_type(&data_type).as_ptr();
                let compare_func = comparer.get_immediate_compare_func(scan_parameters.get_compare_type(), data_type);

                for index in 0..iterations {
                    let current_value_pointer = current_value_pointer.add(index as usize * 16);
                    let compare_result = compare_func(current_value_pointer, immediate_value);

                    encode_results(compare_result);
                }
            } else if scan_parameters.is_relative_comparison() {
                let compare_func = comparer.get_relative_compare_func(scan_parameters.get_compare_type(), data_type);

                for index in 0..iterations {
                    let current_value_pointer = current_value_pointer.add(index as usize * 16);
                    let previous_value_pointer = previous_value_pointer.add(index as usize * 16);
                    let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                    encode_results(compare_result);
                }
            } else if scan_parameters.is_relative_delta_comparison() {
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
