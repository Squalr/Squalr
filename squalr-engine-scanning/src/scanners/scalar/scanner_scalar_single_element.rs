use crate::results::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::comparers::scalar::scanner_scalar_comparer::ScannerScalarComparer;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::scanners::snapshot_scanner::Scanner;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_common::values::data_type::DataType;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
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
    unsafe fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_parameters: &ScanParameters,
        data_type: &DataType,
        _: MemoryAlignment,
    ) -> Vec<SnapshotRegionFilter> {
        let scalar_comparer = ScannerScalarComparer::get_instance();
        let compare_result;

        unsafe {
            if scan_parameters.is_immediate_comparison() {
                let current_value_pointer = snapshot_region.get_current_values_pointer(&snapshot_region_filter);
                let immediate_value_ptr = scan_parameters.deanonymize_type(&data_type).as_ptr();
                let compare_func = scalar_comparer.get_immediate_compare_func(scan_parameters.get_compare_type(), data_type);

                compare_result = compare_func(current_value_pointer, immediate_value_ptr);
            } else if scan_parameters.is_relative_comparison() {
                let current_value_pointer = snapshot_region.get_current_values_pointer(&snapshot_region_filter);
                let previous_value_pointer = snapshot_region.get_previous_values_pointer(&snapshot_region_filter);
                let compare_func = scalar_comparer.get_relative_compare_func(scan_parameters.get_compare_type(), data_type);

                compare_result = compare_func(current_value_pointer, previous_value_pointer);
            } else if scan_parameters.is_immediate_comparison() {
                let current_value_pointer = snapshot_region.get_current_values_pointer(&snapshot_region_filter);
                let previous_value_pointer = snapshot_region.get_previous_values_pointer(&snapshot_region_filter);
                let compare_func = scalar_comparer.get_relative_delta_compare_func(scan_parameters.get_compare_type(), data_type);
                let delta_arg_ptr = scan_parameters.deanonymize_type(&data_type).as_ptr();

                compare_result = compare_func(current_value_pointer, previous_value_pointer, delta_arg_ptr);
            } else {
                panic!("Unrecognized comparison");
            }
        }

        if compare_result {
            return vec![SnapshotRegionFilter::new(
                snapshot_region_filter.get_base_address(),
                snapshot_region_filter.get_region_size(),
            )];
        } else {
            return vec![];
        }
    }
}
