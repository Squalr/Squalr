use crate::structures::data_types::built_in_types::u8::data_type_u8::DataTypeU8;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarComparable;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnDelta;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnImmediate;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnRelative;
use crate::structures::scanning::scan_parameters::ScanParameters;
use std::ptr;

type PrimitiveType = u8;

impl ScalarComparable for DataTypeU8 {
    fn get_compare_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnImmediate {
        Box::new(move |current_value_ptr, immediate_value_ptr| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType) == ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType)
        })
    }

    fn get_compare_not_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnImmediate {
        Box::new(move |current_value_ptr, immediate_value_ptr| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType) != ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType)
        })
    }

    fn get_compare_greater_than(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnImmediate {
        Box::new(move |current_value_ptr, immediate_value_ptr| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType) > ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType)
        })
    }

    fn get_compare_greater_than_or_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnImmediate {
        Box::new(move |current_value_ptr, immediate_value_ptr| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType) >= ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType)
        })
    }

    fn get_compare_less_than(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnImmediate {
        Box::new(move |current_value_ptr, immediate_value_ptr| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType) < ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType)
        })
    }

    fn get_compare_less_than_or_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnImmediate {
        Box::new(move |current_value_ptr, immediate_value_ptr| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType) <= ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType)
        })
    }

    fn get_compare_changed(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative {
        Box::new(move |current_value_ptr, previous_value_ptr| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType) != ptr::read_unaligned(previous_value_ptr as *const PrimitiveType)
        })
    }

    fn get_compare_unchanged(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative {
        Box::new(move |current_value_ptr, previous_value_ptr| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType) == ptr::read_unaligned(previous_value_ptr as *const PrimitiveType)
        })
    }

    fn get_compare_increased(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative {
        Box::new(move |current_value_ptr, previous_value_ptr| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType) > ptr::read_unaligned(previous_value_ptr as *const PrimitiveType)
        })
    }

    fn get_compare_decreased(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative {
        Box::new(move |current_value_ptr, previous_value_ptr| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType) < ptr::read_unaligned(previous_value_ptr as *const PrimitiveType)
        })
    }

    fn get_compare_increased_by(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnDelta {
        Box::new(move |current_value_ptr, previous_value_ptr, delta| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType)
                == ptr::read_unaligned(previous_value_ptr as *const PrimitiveType).wrapping_add(ptr::read_unaligned(delta as *const PrimitiveType))
        })
    }

    fn get_compare_decreased_by(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnDelta {
        Box::new(move |current_value_ptr, previous_value_ptr, delta| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType)
                == ptr::read_unaligned(previous_value_ptr as *const PrimitiveType).wrapping_sub(ptr::read_unaligned(delta as *const PrimitiveType))
        })
    }
}
