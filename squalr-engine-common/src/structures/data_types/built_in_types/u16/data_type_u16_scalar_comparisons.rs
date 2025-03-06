use crate::structures::data_types::built_in_types::u16::data_type_u16::DataTypeU16;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarComparable;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnDelta;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnImmediate;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnRelative;
use crate::structures::scanning::scan_parameters::ScanParameters;
use std::ptr;

type PrimitiveType = u16;

impl ScalarComparable for DataTypeU16 {
    fn get_compare_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnImmediate {
        |current_value_ptr, immediate_value_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let immediate_value = ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType);

            current_value == immediate_value
        }
    }

    fn get_compare_not_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnImmediate {
        |current_value_ptr, immediate_value_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let immediate_value = ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType);

            current_value != immediate_value
        }
    }

    fn get_compare_greater_than(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnImmediate {
        |current_value_ptr, immediate_value_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let immediate_value = ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType);

            current_value > immediate_value
        }
    }

    fn get_compare_greater_than_or_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnImmediate {
        |current_value_ptr, immediate_value_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let immediate_value = ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType);

            current_value >= immediate_value
        }
    }

    fn get_compare_less_than(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnImmediate {
        |current_value_ptr, immediate_value_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let immediate_value = ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType);

            current_value < immediate_value
        }
    }

    fn get_compare_less_than_or_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnImmediate {
        |current_value_ptr, immediate_value_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let immediate_value = ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType);

            current_value <= immediate_value
        }
    }

    fn get_compare_changed(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative {
        |current_value_ptr, previous_value_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let previous_value = ptr::read_unaligned(previous_value_ptr as *const PrimitiveType);

            current_value != previous_value
        }
    }

    fn get_compare_unchanged(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative {
        |current_value_ptr, previous_value_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let previous_value = ptr::read_unaligned(previous_value_ptr as *const PrimitiveType);

            current_value == previous_value
        }
    }

    fn get_compare_increased(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative {
        |current_value_ptr, previous_value_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let previous_value = ptr::read_unaligned(previous_value_ptr as *const PrimitiveType);

            current_value > previous_value
        }
    }

    fn get_compare_decreased(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative {
        |current_value_ptr, previous_value_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let previous_value = ptr::read_unaligned(previous_value_ptr as *const PrimitiveType);

            current_value < previous_value
        }
    }

    fn get_compare_increased_by(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnDelta {
        |current_value_ptr, previous_value_ptr, delta_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let previous_value = ptr::read_unaligned(previous_value_ptr as *const PrimitiveType);
            let delta_value = ptr::read_unaligned(delta_ptr as *const PrimitiveType);

            current_value == previous_value.wrapping_add(delta_value)
        }
    }

    fn get_compare_decreased_by(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnDelta {
        |current_value_ptr, previous_value_ptr, delta_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let previous_value = ptr::read_unaligned(previous_value_ptr as *const PrimitiveType);
            let delta_value = ptr::read_unaligned(delta_ptr as *const PrimitiveType);

            current_value == previous_value.wrapping_sub(delta_value)
        }
    }
}
