use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::comparers::scalar::scanner_scalar_comparer::ScannerScalarComparer;
use crate::scanners::encoders::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use std::sync::Once;

pub struct ScannerScalarEncoder {}

impl ScannerScalarEncoder {
    fn new() -> Self {
        Self {}
    }

    pub fn get_instance() -> &'static ScannerScalarEncoder {
        static mut INSTANCE: Option<ScannerScalarEncoder> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = ScannerScalarEncoder::new();
                INSTANCE = Some(instance);
            });

            return INSTANCE.as_ref().unwrap_unchecked();
        }
    }

    /// Scans a region of memory defined by the given parameters. Uses a run length encoding algorithm to only create the scan
    /// result when a false comparison is encountered (or if out of bytes to scan).
    pub fn encode(
        &self,
        current_value_pointer: *const u8,
        previous_value_pointer: *const u8,
        scan_parameters: &ScanParameters,
        scan_filter_parameters: &ScanFilterParameters,
        base_address: u64,
        element_count: u64,
    ) -> Vec<SnapshotRegionFilter> {
        let comparer = ScannerScalarComparer::get_instance();
        let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
        let data_type = scan_filter_parameters.get_data_type();
        let data_type_size = data_type.get_size_in_bytes();
        let memory_alignment = scan_filter_parameters.get_memory_alignment_or_default() as u64;
        let data_type_size_padding = data_type_size.saturating_sub(memory_alignment);

        unsafe {
            // Run length encoding for the scan results
            let mut encode_results = |compare_result: bool| {
                if compare_result {
                    run_length_encoder.encode_range(memory_alignment);
                } else {
                    run_length_encoder.finalize_current_encode_data_size_padded(memory_alignment, data_type_size_padding);
                }
            };

            if scan_parameters.is_immediate_comparison() {
                let immediate_value_ptr = scan_parameters.deanonymize_type(&data_type).as_ptr();
                let compare_func = comparer.get_immediate_compare_func(scan_parameters.get_compare_type(), &data_type);

                for index in 0..element_count {
                    let current_value_pointer = current_value_pointer.add(index as usize * memory_alignment as usize);
                    let result = compare_func(current_value_pointer, immediate_value_ptr);

                    encode_results(result);
                }
            } else if scan_parameters.is_relative_comparison() {
                let compare_func = comparer.get_relative_compare_func(scan_parameters.get_compare_type(), data_type);

                for index in 0..element_count {
                    let current_value_pointer = current_value_pointer.add(index as usize * memory_alignment as usize);
                    let previous_value_pointer = previous_value_pointer.add(index as usize * memory_alignment as usize);
                    let result = compare_func(current_value_pointer, previous_value_pointer);

                    encode_results(result);
                }
            } else if scan_parameters.is_relative_delta_comparison() {
                let compare_func = comparer.get_relative_delta_compare_func(scan_parameters.get_compare_type(), data_type);
                let delta_arg_ptr = scan_parameters.deanonymize_type(&data_type).as_ptr();

                for index in 0..element_count {
                    let current_value_pointer = current_value_pointer.add(index as usize * memory_alignment as usize);
                    let previous_value_pointer = previous_value_pointer.add(index as usize * memory_alignment as usize);
                    let result = compare_func(current_value_pointer, previous_value_pointer, delta_arg_ptr);

                    encode_results(result);
                }
            } else {
                panic!("Unrecognized comparison");
            }
        }

        run_length_encoder.finalize_current_encode_data_size_padded(memory_alignment, data_type_size_padding);

        return run_length_encoder.take_result_regions();
    }
}
