use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::snapshots::snapshot_region::SnapshotRegion;
use crate::scanners::comparers::scalar::scanner_scalar_comparer::ScannerScalarComparer;
use crate::scanners::comparers::snapshot_scanner::Scanner;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use std::borrow::BorrowMut;
use std::sync::Once;

pub struct ScannerScalarSingleElement {
}

impl ScannerScalarSingleElement {
    fn new(
    ) -> Self {
        Self { }
    }
    
    pub fn get_instance(
    ) -> &'static ScannerScalarSingleElement {
        static mut INSTANCE: Option<ScannerScalarSingleElement> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = ScannerScalarSingleElement::new();
                INSTANCE = Some(instance);
            });

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
        scan_filter_parameters: &ScanFilterParameters,
    ) -> Vec<SnapshotRegionFilter> {
        let data_type = scan_filter_parameters.get_data_type();
        let scalar_comparer = ScannerScalarComparer::get_instance();
        let compare_result;
        let memory_load_func = data_type.get_load_memory_function_ptr();
        
        unsafe {
            if scan_parameters.is_immediate_comparison() {
                let current_value_pointer = snapshot_region.get_current_values_pointer(&snapshot_region_filter);
                let mut current_value = data_type.to_default_value();
                let immediate_value = scan_parameters.deanonymize_type(&data_type).unwrap(); // TODO: Handle and complain
                let compare_func = scalar_comparer.get_immediate_compare_func(scan_parameters.get_compare_type());

                memory_load_func(current_value.borrow_mut(), current_value_pointer);
                compare_result = compare_func(&current_value, &immediate_value);
            } else if scan_parameters.is_relative_comparison() {
                let current_value_pointer = snapshot_region.get_current_values_pointer(&snapshot_region_filter);
                let previous_value_pointer = snapshot_region.get_previous_values_pointer(&snapshot_region_filter);
                let mut current_value = data_type.to_default_value();
                let mut previous_value = data_type.to_default_value();
                let compare_func = scalar_comparer.get_relative_compare_func(scan_parameters.get_compare_type());

                memory_load_func(current_value.borrow_mut(), current_value_pointer);
                memory_load_func(previous_value.borrow_mut(), previous_value_pointer);
                compare_result = compare_func(&current_value, &previous_value);
            } else if scan_parameters.is_immediate_comparison() {
                let current_value_pointer = snapshot_region.get_current_values_pointer(&snapshot_region_filter);
                let previous_value_pointer = snapshot_region.get_previous_values_pointer(&snapshot_region_filter);
                let mut current_value = data_type.to_default_value();
                let mut previous_value = data_type.to_default_value();
                let compare_func = scalar_comparer.get_relative_delta_compare_func(scan_parameters.get_compare_type());
                let delta_arg = scan_parameters.deanonymize_type(&data_type).unwrap(); // TODO: Handle and complain
    
                memory_load_func(current_value.borrow_mut(), current_value_pointer);
                memory_load_func(previous_value.borrow_mut(), previous_value_pointer);
                compare_result = compare_func(&current_value, &previous_value, &delta_arg);
            } else {
                panic!("Unrecognized comparison");
            }
        }

        if compare_result {
            return vec![SnapshotRegionFilter::new(snapshot_region_filter.get_base_address(), snapshot_region_filter.get_region_size())];
        } else {
            return vec![];
        }
    }
}
