use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::scanners::snapshot_scanner::Scanner;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_common::structures::data_types::data_type::DataType;
use squalr_engine_common::structures::memory_alignment::MemoryAlignment;
use squalr_engine_common::structures::scanning::scan_compare_type::ScanCompareType;
use std::sync::Once;

pub struct ScannerScalarSingleElement {}

impl ScannerScalarSingleElement {
    fn new() -> Self {
        Self {}
    }

    pub fn get_instance() -> &'static ScannerScalarSingleElement {
        static mut INSTANCE: Option<ScannerScalarSingleElement> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = ScannerScalarSingleElement::new();
                INSTANCE = Some(instance);
            });

            #[allow(static_mut_refs)]
            return INSTANCE.as_ref().unwrap_unchecked();
        }
    }
}

/// Implements a scalar (ie CPU bound, non-SIMD) scanner which only scans a single element of memory (ie only containing 1 data type).
impl Scanner for ScannerScalarSingleElement {
    fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_parameters: &ScanParameters,
        data_type: &Box<dyn DataType>,
        _: MemoryAlignment,
    ) -> Vec<SnapshotRegionFilter> {
        let compare_result;

        unsafe {
            match scan_parameters.get_compare_type() {
                ScanCompareType::Immediate(scan_compare_type_immediate) => {
                    let current_value_pointer = snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter);
                    let immediate_value = scan_parameters.deanonymize_type(&data_type);
                    let immediate_value_ptr = immediate_value.as_ptr();
                    let compare_func = data_type.get_immediate_compare_func(scan_parameters.get_compare_type(), &data_type);

                    compare_result = compare_func(current_value_pointer, immediate_value_ptr);
                }
                ScanCompareType::Relative(scan_compare_type_relative) => {
                    let current_value_pointer = snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter);
                    let previous_value_pointer = snapshot_region.get_previous_values_filter_pointer(&snapshot_region_filter);
                    let compare_func = data_type.get_relative_compare_func(scan_parameters.get_compare_type(), data_type);

                    compare_result = compare_func(current_value_pointer, previous_value_pointer);
                }
                ScanCompareType::Delta(scan_compare_type_delta) => {
                    let current_value_pointer = snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter);
                    let previous_value_pointer = snapshot_region.get_previous_values_filter_pointer(&snapshot_region_filter);
                    let delta_arg = scan_parameters.deanonymize_type(&data_type);
                    let delta_arg_ptr = delta_arg.as_ptr();
                    let compare_func = data_type.get_relative_delta_compare_func(scan_parameters.get_compare_type(), data_type);

                    compare_result = compare_func(current_value_pointer, previous_value_pointer, delta_arg_ptr);
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
