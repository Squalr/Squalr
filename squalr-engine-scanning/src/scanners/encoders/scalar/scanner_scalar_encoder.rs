use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::encoders::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use squalr_engine_common::structures::data_types::data_type::DataType;
use squalr_engine_common::structures::memory_alignment::MemoryAlignment;
use squalr_engine_common::structures::scanning::scan_compare_type::ScanCompareType;

pub struct ScannerScalarEncoder {}

impl ScannerScalarEncoder {
    /// Scans a region of memory defined by the given parameters. Uses a run length encoding algorithm to only create the scan
    /// result when a false comparison is encountered (or if out of bytes to scan).
    pub fn scalar_encode(
        current_value_pointer: *const u8,
        previous_value_pointer: *const u8,
        scan_parameters: &ScanParameters,
        data_type: &Box<dyn DataType>,
        memory_alignment: MemoryAlignment,
        base_address: u64,
        element_count: u64,
    ) -> Vec<SnapshotRegionFilter> {
        let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
        let data_type_size = data_type.get_size_in_bytes();
        let memory_alignment = memory_alignment as u64;
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

            match scan_parameters.get_compare_type() {
                ScanCompareType::Immediate(scan_compare_type_immediate) => {
                    let immediate_value = scan_parameters.deanonymize_type(&data_type);
                    let immediate_value_ptr = immediate_value.as_ptr();
                    let compare_func = data_type.get_immediate_compare_func(scan_parameters.get_compare_type(), &data_type);

                    for index in 0..element_count {
                        let current_value_pointer = current_value_pointer.add(index as usize * memory_alignment as usize);
                        let result = compare_func(current_value_pointer, immediate_value_ptr);

                        encode_results(result);
                    }
                }
                ScanCompareType::Relative(scan_compare_type_relative) => {
                    let compare_func = data_type.get_relative_compare_func(scan_parameters.get_compare_type(), data_type);

                    for index in 0..element_count {
                        let current_value_pointer = current_value_pointer.add(index as usize * memory_alignment as usize);
                        let previous_value_pointer = previous_value_pointer.add(index as usize * memory_alignment as usize);
                        let result = compare_func(current_value_pointer, previous_value_pointer);

                        encode_results(result);
                    }
                }
                ScanCompareType::Delta(scan_compare_type_delta) => {
                    let compare_func = data_type.get_relative_delta_compare_func(scan_parameters.get_compare_type(), data_type);
                    let delta_arg = scan_parameters.deanonymize_type(&data_type);
                    let delta_arg_ptr = delta_arg.as_ptr();

                    for index in 0..element_count {
                        let current_value_pointer = current_value_pointer.add(index as usize * memory_alignment as usize);
                        let previous_value_pointer = previous_value_pointer.add(index as usize * memory_alignment as usize);
                        let result = compare_func(current_value_pointer, previous_value_pointer, delta_arg_ptr);

                        encode_results(result);
                    }
                }
            }
        }

        run_length_encoder.finalize_current_encode_data_size_padded(memory_alignment, data_type_size_padding);
        run_length_encoder.take_result_regions()
    }
}
