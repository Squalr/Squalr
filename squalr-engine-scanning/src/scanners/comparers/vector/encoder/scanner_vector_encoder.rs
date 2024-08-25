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
                vector_comparer: &ScannerVectorComparer::<$vector_bit_size>,
                true_mask: $simd_type,
            ) -> Vec<SnapshotRegionFilter> {
                let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
                let data_type = scan_filter_parameters.get_data_type();
                let data_type_size = data_type.get_size_in_bytes();
                let comparisons_per_vector = ($vector_byte_size / data_type_size);
                let iterations = element_count / comparisons_per_vector;
                let remainder_elements = element_count % comparisons_per_vector;
                let false_mask = <$simd_type>::splat(0);

                unsafe {
                    if scan_parameters.is_immediate_comparison() {
                        let immediate_value = scan_parameters.deanonymize_type(&data_type).as_ptr();
                        let compare_func = vector_comparer.get_immediate_compare_func(scan_parameters.get_compare_type(), data_type);

                        // Compare as many full vectors as we can
                        for index in 0..iterations {
                            let current_value_pointer = current_value_pointer.add(index as usize * $vector_byte_size);
                            let compare_result = compare_func(current_value_pointer, immediate_value);

                            self.encode_results(&
                                compare_result,
                                &mut run_length_encoder,
                                data_type_size,
                                true_mask,
                                false_mask
                            );
                        }

                        // Handle remainder elements
                        if remainder_elements > 0 {
                            let current_value_pointer = current_value_pointer.add((iterations as usize * $vector_byte_size) - $vector_byte_size);
                            let compare_result = compare_func(current_value_pointer, immediate_value);

                            self.encode_remainder_results(
                                &compare_result,
                                &mut run_length_encoder,
                                data_type_size,
                                remainder_elements,
                            );
                        }

                    } else if scan_parameters.is_relative_comparison() {
                        let compare_func = vector_comparer.get_relative_compare_func(scan_parameters.get_compare_type(), data_type);

                        // Compare as many full vectors as we can
                        for index in 0..iterations {
                            let current_value_pointer = current_value_pointer.add(index as usize * $vector_byte_size);
                            let previous_value_pointer = previous_value_pointer.add(index as usize * $vector_byte_size);
                            let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                            self.encode_results(&
                                compare_result,
                                &mut run_length_encoder,
                                data_type_size,
                                true_mask,
                                false_mask
                            );
                        }

                        // Handle remainder elements
                        if remainder_elements > 0 {
                            let current_value_pointer = current_value_pointer.add((iterations as usize * $vector_byte_size) - $vector_byte_size);
                            let previous_value_pointer = previous_value_pointer.add((iterations as usize * $vector_byte_size) - $vector_byte_size);
                            let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                            self.encode_remainder_results(
                                &compare_result,
                                &mut run_length_encoder,
                                data_type_size,
                                remainder_elements,
                            );
                        }
                    } else if scan_parameters.is_relative_delta_comparison() {
                        let compare_func = vector_comparer.get_immediate_compare_func(scan_parameters.get_compare_type(), data_type);
                        let delta_arg = scan_parameters.deanonymize_type(&data_type).as_ptr();

                        // Compare as many full vectors as we can
                        for index in 0..iterations {
                            let current_value_pointer = current_value_pointer.add(index as usize * $vector_byte_size);
                            let compare_result = compare_func(current_value_pointer, delta_arg);

                            self.encode_results(
                                &compare_result,
                                &mut run_length_encoder,
                                data_type_size,
                                true_mask,
                                false_mask
                            );
                        }

                        // Handle remainder elements
                        if remainder_elements > 0 {
                            let current_value_pointer = current_value_pointer.add((iterations as usize * $vector_byte_size) - $vector_byte_size);
                            let compare_result = compare_func(current_value_pointer, delta_arg);

                            self.encode_remainder_results(
                                &compare_result,
                                &mut run_length_encoder,
                                data_type_size,
                                remainder_elements,
                            );
                        }
                    } else {
                        panic!("Unrecognized comparison");
                    }
                }

                run_length_encoder.finalize_current_encode(0);

                return run_length_encoder.result_regions;
            }

            #[inline(always)]
            fn encode_results(
                &self,
                compare_result: &Simd<u8, $vector_byte_size>,
                run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
                data_type_size: u64,
                true_mask: $simd_type,
                false_mask: $simd_type,
            ) {
                // Optimization: Check if all scan results are true. This helps substantially when scanning for common values like 0.
                if compare_result.eq(&true_mask) {
                    run_length_encoder.encode_range(($vector_byte_size) as u64);
                // Optimization: Check if all scan results are false. This is also a very common result, and speeds up scans.
                } else if compare_result.eq(&false_mask) {
                    run_length_encoder.finalize_current_encode(($vector_byte_size) as u64);
                // Otherwise, there is a mix of true/false results that need to be processed manually.
                } else {
                    for byte_index in (0..$vector_byte_size).step_by(data_type_size as usize) {
                        if compare_result[byte_index] != 0 {
                            run_length_encoder.encode_range(data_type_size);
                        } else {
                            run_length_encoder.finalize_current_encode(data_type_size);
                        }
                    }
                }
            }

            #[inline(always)]
            fn encode_remainder_results(
                &self,
                compare_result: &Simd<u8, $vector_byte_size>,
                run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
                data_type_size: u64,
                remainder_elements: u64,
            ) {
                let start_byte_index = ($vector_byte_size - remainder_elements as usize * data_type_size as usize) as usize;

                for byte_index in (start_byte_index..$vector_byte_size).step_by(data_type_size as usize) {
                    if compare_result[byte_index] != 0 {
                        run_length_encoder.encode_range(data_type_size);
                    } else {
                        run_length_encoder.finalize_current_encode(data_type_size);
                    }
                }
            }
        }
    };
}

// Create implementations for various SIMD hardware vector sizes.
impl_scanner_vector_encoder!(128, u8x16, 16);
impl_scanner_vector_encoder!(256, u8x32, 32);
impl_scanner_vector_encoder!(512, u8x64, 64);
