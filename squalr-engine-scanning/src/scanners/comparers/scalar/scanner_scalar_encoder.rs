use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::comparers::scalar::scanner_scalar_comparer::ScannerScalarComparer;
use crate::scanners::comparers::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use crate::scanners::constraints::scan_constraint::ScanConstraint;
use crate::scanners::constraints::scan_filter_constraint::ScanFilterConstraint;
use std::borrow::BorrowMut;
use std::sync::Once;

pub struct ScannerScalarEncoder {
}

impl ScannerScalarEncoder {
    fn new(
    ) -> Self {
        Self { }
    }
    
    pub fn get_instance(
    ) -> &'static ScannerScalarEncoder {
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

    pub fn encode(
        &self,
        current_value_pointer: *const u8,
        previous_value_pointer: *const u8,
        scan_constraint: &ScanConstraint,
        filter_constraint: &ScanFilterConstraint,
        base_address: u64,
        element_count: u64,
    ) -> Vec<SnapshotRegionFilter> {
        let comparer = ScannerScalarComparer::get_instance();
        let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
        let data_type = filter_constraint.get_data_type();
        let data_type_size = data_type.size_in_bytes();
        let memory_alignment = filter_constraint.get_memory_alignment_or_default(data_type);
        let memory_load_func = data_type.get_load_memory_function_ptr();

        unsafe {
            if scan_constraint.is_immediate_constraint() {
                let mut current_value = data_type.to_default_value();
                let mut immediate_value = scan_constraint.get_constraint_value().unwrap().clone();
                let current_value = current_value.borrow_mut();
                let immediate_value = immediate_value.borrow_mut();
                let compare_func = comparer.get_immediate_compare_func(scan_constraint.get_constraint_type());

                for index in 0..element_count {
                    let current_value_pointer = current_value_pointer.add(index as usize * memory_alignment as usize);

                    memory_load_func(current_value, current_value_pointer);

                    if compare_func(current_value, immediate_value) {
                        run_length_encoder.encode_range(memory_alignment);
                    } else {
                        run_length_encoder.finalize_current_encode_unchecked(memory_alignment, data_type_size);
                    }
                }
            } else if scan_constraint.is_relative_constraint() {
                let mut current_value = data_type.to_default_value();
                let mut previous_value = data_type.to_default_value();
                let current_value = current_value.borrow_mut();
                let previous_value = previous_value.borrow_mut();
                let compare_func = comparer.get_relative_compare_func(scan_constraint.get_constraint_type());

                for index in 0..element_count {
                    let current_value_pointer = current_value_pointer.add(index as usize * memory_alignment as usize);
                    let previous_value_pointer = previous_value_pointer.add(index as usize * memory_alignment as usize);

                    memory_load_func(current_value, current_value_pointer);
                    memory_load_func(previous_value, previous_value_pointer);

                    if compare_func(
                        current_value,
                        previous_value,
                    ) {
                        run_length_encoder.encode_range(memory_alignment);
                    } else {
                        run_length_encoder.finalize_current_encode_unchecked(memory_alignment, data_type_size);
                    }
                }
            } else if scan_constraint.is_immediate_constraint() {
                let mut current_value = data_type.to_default_value();
                let mut previous_value = data_type.to_default_value();
                let current_value = current_value.borrow_mut();
                let previous_value = previous_value.borrow_mut();
                let compare_func = comparer.get_relative_delta_compare_func(scan_constraint.get_constraint_type());
                let delta_arg = scan_constraint.get_constraint_value().unwrap(); // TODO: Handle

                for index in 0..element_count {
                    let current_value_pointer = current_value_pointer.add(index as usize * memory_alignment as usize);
                    let previous_value_pointer = previous_value_pointer.add(index as usize * memory_alignment as usize);

                    memory_load_func(current_value, current_value_pointer);
                    memory_load_func(previous_value, previous_value_pointer);

                    if compare_func(
                        current_value,
                        previous_value,
                        delta_arg,
                    ) {
                        run_length_encoder.encode_range(memory_alignment);
                    } else {
                        run_length_encoder.finalize_current_encode_unchecked(memory_alignment, data_type_size);
                    }
                }
            } else {
                panic!("Unrecognized constraint");
            }
        }

        run_length_encoder.finalize_current_encode_unchecked(memory_alignment, data_type_size);
        
        return run_length_encoder.result_regions;
    }
}
