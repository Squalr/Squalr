use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::encoders::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use squalr_engine_api::structures::scanning::scan_compare_type::ScanCompareType;
use squalr_engine_api::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use squalr_engine_api::structures::scanning::scan_parameters_local::ScanParametersLocal;

pub struct ScannerScalarEncoder {}

impl ScannerScalarEncoder {
    /// Scans a region of memory defined by the given parameters. Uses a run length encoding algorithm to only create the scan
    /// result when a false comparison is encountered (or if out of bytes to scan).
    pub fn scalar_encode(
        current_value_pointer: *const u8,
        previous_value_pointer: *const u8,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
        base_address: u64,
        element_count: u64,
    ) -> Vec<SnapshotRegionFilter> {
        let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
        let data_type = scan_parameters_local.get_data_type();
        let memory_alignment = scan_parameters_local.get_memory_alignment_or_default();
        let data_type_size = data_type.get_size_in_bytes();
        let memory_alignment = memory_alignment as u64;
        let data_type_size_padding = data_type_size.saturating_sub(memory_alignment);

        unsafe {
            // Run length encoding for the scan results.
            let mut encode_results = |compare_result: bool| {
                if compare_result {
                    run_length_encoder.encode_range(memory_alignment);
                } else {
                    run_length_encoder.finalize_current_encode_data_size_padded(memory_alignment, data_type_size_padding);
                }
            };

            match scan_parameters_global.get_compare_type() {
                ScanCompareType::Immediate(scan_compare_type_immediate) => {
                    if let Some(compare_func) =
                        data_type.get_scalar_compare_func_immediate(&scan_compare_type_immediate, scan_parameters_global, scan_parameters_local)
                    {
                        for index in 0..element_count {
                            let current_value_pointer = current_value_pointer.add(index as usize * memory_alignment as usize);
                            let result = compare_func(current_value_pointer);

                            encode_results(result);
                        }
                    }
                }
                ScanCompareType::Relative(scan_compare_type_relative) => {
                    if let Some(compare_func) =
                        data_type.get_scalar_compare_func_relative(&scan_compare_type_relative, scan_parameters_global, scan_parameters_local)
                    {
                        for index in 0..element_count {
                            let current_value_pointer = current_value_pointer.add(index as usize * memory_alignment as usize);
                            let previous_value_pointer = previous_value_pointer.add(index as usize * memory_alignment as usize);
                            let result = compare_func(current_value_pointer, previous_value_pointer);

                            encode_results(result);
                        }
                    }
                }
                ScanCompareType::Delta(scan_compare_type_delta) => {
                    if let Some(compare_func) = data_type.get_scalar_compare_func_delta(&scan_compare_type_delta, scan_parameters_global, scan_parameters_local)
                    {
                        for index in 0..element_count {
                            let current_value_pointer = current_value_pointer.add(index as usize * memory_alignment as usize);
                            let previous_value_pointer = previous_value_pointer.add(index as usize * memory_alignment as usize);
                            let result = compare_func(current_value_pointer, previous_value_pointer);

                            encode_results(result);
                        }
                    }
                }
            }
        }

        run_length_encoder.finalize_current_encode_data_size_padded(memory_alignment, data_type_size_padding);
        run_length_encoder.take_result_regions()
    }
}
