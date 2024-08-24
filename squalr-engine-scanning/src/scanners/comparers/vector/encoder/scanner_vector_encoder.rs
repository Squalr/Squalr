use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::comparers::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use crate::scanners::comparers::vector::encoder::scanner_vector_comparer::ScannerVectorComparer;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use std::simd::{Simd, u8x16, u8x32, u8x64};
use std::sync::Once;

pub struct ScannerVectorEncoder<const VECTOR_SIZE_BITS: usize>;

macro_rules! impl_scanner_vector_encoder {
    ($vector_bit_size:expr, $simd_type:ty, $vector_byte_size:expr) => {
        impl ScannerVectorEncoder<$vector_bit_size> {
            pub fn get_instance() -> &'static ScannerVectorEncoder<$vector_bit_size> {
                static mut INSTANCE: Option<ScannerVectorEncoder<$vector_bit_size>> = None;
                static INIT: Once = Once::new();

                unsafe {
                    INIT.call_once(|| {
                        let instance = ScannerVectorEncoder::<$vector_bit_size>::new();
                        INSTANCE = Some(instance);
                    });

                    INSTANCE.as_ref().unwrap_unchecked()
                }
            }

            fn new() -> Self {
                Self {}
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
                let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
                let comparer = ScannerVectorComparer::<$vector_bit_size>::get_instance();
                let data_type = scan_filter_parameters.get_data_type();
                let data_type_size = data_type.size_in_bytes();
                let memory_alignment = scan_filter_parameters.get_memory_alignment_or_default() as u64;
                let comparisons_per_vector = ($vector_byte_size / data_type_size);
                let iterations = element_count / comparisons_per_vector;
                let remainder_elements = element_count % comparisons_per_vector;
                let true_mask = <$simd_type>::splat(0xFF);
                let false_mask = <$simd_type>::splat(0);

                unsafe {
                    let mut encode_results = |compare_result: Simd<u8, $vector_byte_size>, bytes_to_process: usize| {
                        if compare_result.eq(&true_mask) {
                            run_length_encoder.encode_range(bytes_to_process as u64);
                        } else if compare_result.eq(&false_mask) {
                            run_length_encoder.finalize_current_encode_unsized(bytes_to_process as u64);
                        } else {
                            for byte_index in (0..bytes_to_process).step_by(data_type_size as usize) {
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
                            let current_value_pointer = current_value_pointer.add(index as usize * $vector_byte_size);
                            let compare_result = compare_func(current_value_pointer, immediate_value);

                            encode_results(compare_result, $vector_byte_size);
                        }

                        // Handle remainder
                        if remainder_elements > 0 {
                            let current_value_pointer = current_value_pointer.add(iterations as usize * $vector_byte_size);
                            let compare_result = compare_func(current_value_pointer, immediate_value);

                            encode_results(compare_result, remainder_elements as usize * data_type_size as usize);
                        }
                    } else if scan_parameters.is_relative_comparison() {
                        let compare_func = comparer.get_relative_compare_func(scan_parameters.get_compare_type(), data_type);

                        for index in 0..iterations {
                            let current_value_pointer = current_value_pointer.add(index as usize * $vector_byte_size);
                            let previous_value_pointer = previous_value_pointer.add(index as usize * $vector_byte_size);
                            let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                            encode_results(compare_result, $vector_byte_size);
                        }

                        // Handle remainder
                        if remainder_elements > 0 {
                            let current_value_pointer = current_value_pointer.add(iterations as usize * $vector_byte_size);
                            let previous_value_pointer = previous_value_pointer.add(iterations as usize * $vector_byte_size);
                            let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                            encode_results(compare_result, remainder_elements as usize * data_type_size as usize);
                        }
                    } else if scan_parameters.is_relative_delta_comparison() {
                        let compare_func = comparer.get_immediate_compare_func(scan_parameters.get_compare_type(), data_type);
                        let delta_arg = scan_parameters.deanonymize_type(&data_type).as_ptr();

                        for index in 0..iterations {
                            let current_value_pointer = current_value_pointer.add(index as usize * memory_alignment as usize * $vector_byte_size * data_type_size as usize);
                            let compare_result = compare_func(current_value_pointer, delta_arg);

                            encode_results(compare_result, $vector_byte_size);
                        }

                        // Handle remainder
                        if remainder_elements > 0 {
                            let current_value_pointer = current_value_pointer.add(iterations as usize * memory_alignment as usize * $vector_byte_size * data_type_size as usize);
                            let compare_result = compare_func(current_value_pointer, delta_arg);

                            encode_results(compare_result, remainder_elements as usize * data_type_size as usize);
                        }
                    } else {
                        panic!("Unrecognized comparison");
                    }
                }

                run_length_encoder.finalize_current_encode_unsized(0);

                return run_length_encoder.result_regions;
            }
        }
    };
}

// Implement for different vector sizes and types
impl_scanner_vector_encoder!(128, u8x16, 16);
impl_scanner_vector_encoder!(256, u8x32, 32);
impl_scanner_vector_encoder!(512, u8x64, 64);
