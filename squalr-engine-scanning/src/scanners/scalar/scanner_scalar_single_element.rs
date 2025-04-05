use crate::scanners::snapshot_scanner::Scanner;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use squalr_engine_api::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
use squalr_engine_api::structures::scanning::parameters::mapped::mapped_scan_parameters::MappedScanParameters;

pub struct ScannerScalarSingleElement {}

/// Implements a scalar (ie CPU bound, non-SIMD) scanner which only scans a single element of memory (ie only containing 1 data type).
impl Scanner for ScannerScalarSingleElement {
    fn scan_region(
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_parameters: &MappedScanParameters,
    ) -> Vec<SnapshotRegionFilter> {
        let mut compare_result = false;
        let data_type = scan_parameters.get_data_type();

        match scan_parameters.get_compare_type() {
            ScanCompareType::Immediate(scan_compare_type_immediate) => {
                if let Some(compare_func) = data_type.get_scalar_compare_func_immediate(&scan_compare_type_immediate, scan_parameters) {
                    let current_value_pointer = snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter);

                    compare_result = compare_func(current_value_pointer);
                }
            }
            ScanCompareType::Relative(scan_compare_type_relative) => {
                if let Some(compare_func) = data_type.get_scalar_compare_func_relative(&scan_compare_type_relative, scan_parameters) {
                    let current_value_pointer = snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter);
                    let previous_value_pointer = snapshot_region.get_previous_values_filter_pointer(&snapshot_region_filter);

                    compare_result = compare_func(current_value_pointer, previous_value_pointer);
                }
            }
            ScanCompareType::Delta(scan_compare_type_delta) => {
                if let Some(compare_func) = data_type.get_scalar_compare_func_delta(&scan_compare_type_delta, scan_parameters) {
                    let current_value_pointer = snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter);
                    let previous_value_pointer = snapshot_region.get_previous_values_filter_pointer(&snapshot_region_filter);

                    compare_result = compare_func(current_value_pointer, previous_value_pointer);
                }
            }
        }

        if compare_result {
            vec![SnapshotRegionFilter::new(
                snapshot_region_filter.get_base_address(),
                snapshot_region_filter.get_region_size(),
            )]
        } else {
            vec![]
        }
    }
}
