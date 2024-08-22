use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::comparers::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use crate::scanners::comparers::vector::encoder::scanner_vector_comparer_128::ScannerVectorComparer;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use std::sync::Once;

/*
Brain dump.

It looks like we have a LOT more options than C# as far as SIMD implementation, although as caveat is that we (as far as I can tell) will not be able
to easily genericize all SIMD vector legnths.

However, When we perform a scan, we can extract the packed byte results.

This means if scanning for a double with 16 byte vector sizes, we could collapse this vector to 2 bytes containing the result.

This means we are free to, instead of scanning two doubles, scan 16 doubles (ie elements == the full vector size)

We then scan all 16 in an unrolled loop, pack the results together, and ship them back to the encoder as 16 bools.

Although, if I were a madman, I could potentially figure out how to pack 128 scan results into a single vector via bit masking.

Of course this would hit some bad perf issues on scan misses, buuuut considering 70-80% of process memory is all 0s, and this is the bottleneck

Having 128-512 simultaneous encodes in a single scan loop would be fucking insane.

C# never had this flexibility, so I never considered it.

Maybe, just maybe, upon failures we can do per-byte processing that passes the byte to a jump table of 256 entries that do encodes 8 at a time.
This might be stupid and I could be an idiot, just thinking out loud.

*/

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
        let comparer = ScannerVectorComparer::get_instance();
        let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
        let data_type = scan_filter_parameters.get_data_type();
        let data_type_size = data_type.size_in_bytes();
        let memory_alignment = scan_filter_parameters.get_memory_alignment_or_default() as u64;
        let iterations = element_count / 16;
        
        unsafe {
            if scan_parameters.is_immediate_comparison() {
                let immediate_value = scan_parameters.deanonymize_type(&data_type).as_ptr();
                let compare_func = comparer.get_immediate_compare_func(scan_parameters.get_compare_type(), data_type);

                for index in 0..iterations {
                    let current_value_pointer = current_value_pointer.add(index as usize * 16 * data_type_size as usize);
                    let compare_result: std::simd::Mask<i8, 16> = compare_func(current_value_pointer, immediate_value);

                    if compare_result.all() {
                        run_length_encoder.encode_range(16);
                    } else if !compare_result.any() {
                        run_length_encoder.finalize_current_encode_unsized(16);
                    } else {
                        let raw_results = compare_result.to_bitmask_vector().to_array();
                        for result in raw_results {
                            if result != 0 {
                                run_length_encoder.encode_range(1);
                            } else {
                                run_length_encoder.finalize_current_encode(1, data_type_size);
                            }
                        }
                    }
                }
            } else if scan_parameters.is_relative_comparison() {
                let compare_func = comparer.get_relative_compare_func(scan_parameters.get_compare_type(), data_type);

                for index in 0..iterations {
                    let current_value_pointer = current_value_pointer.add(index as usize * memory_alignment as usize * 16 * data_type_size as usize);
                    let previous_value_pointer = previous_value_pointer.add(index as usize * memory_alignment as usize * 16 * data_type_size as usize);
                    let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                    if compare_result.all() {
                        run_length_encoder.encode_range(16);
                    } else if !compare_result.any() {
                        run_length_encoder.finalize_current_encode(16, data_type_size);
                    } else {
                        for result in compare_result.to_int().as_array() {
                            if *result != 0 {
                                run_length_encoder.encode_range(memory_alignment);
                            } else {
                                run_length_encoder.finalize_current_encode(memory_alignment, data_type_size);
                            }
                        }
                    }
                }
            } else if scan_parameters.is_immediate_comparison() {
                let compare_func = comparer.get_immediate_compare_func(scan_parameters.get_compare_type(), data_type);
                let delta_arg = scan_parameters.deanonymize_type(&data_type).as_ptr();

                for index in 0..iterations {
                    let current_value_pointer = current_value_pointer.add(index as usize * memory_alignment as usize * 16 * data_type_size as usize);
                    let compare_result = compare_func(current_value_pointer, delta_arg);

                    if compare_result.all() {
                        run_length_encoder.encode_range(16);
                    } else if !compare_result.any() {
                        run_length_encoder.finalize_current_encode(16, data_type_size);
                    } else {
                        run_length_encoder.finalize_current_encode(16, data_type_size);
                        for result in compare_result.to_int().as_array() {
                            if *result != 0 {
                                run_length_encoder.encode_range(memory_alignment);
                            } else {
                                run_length_encoder.finalize_current_encode(memory_alignment, data_type_size);
                            }
                        }
                    }
                }
            } else {
                panic!("Unrecognized comparison");
            }
        }

        run_length_encoder.finalize_current_encode_unsized(memory_alignment);
        
        return run_length_encoder.result_regions;
    }
}
