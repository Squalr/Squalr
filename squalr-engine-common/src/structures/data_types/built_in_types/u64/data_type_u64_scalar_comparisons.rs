use crate::structures::data_types::built_in_types::u64::data_type_u64::DataTypeU64;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarComparable;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnDelta;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnImmediate;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnRelative;
use std::ptr;

type PrimitiveType = u64;

impl ScalarComparable for DataTypeU64 {
    fn get_compare_equal(&self) -> ScalarCompareFnImmediate {
        |current_value_ptr, immediate_value_ptr| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType) == ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType)
        }
    }

    fn get_compare_not_equal(&self) -> ScalarCompareFnImmediate {
        |current_value_ptr, immediate_value_ptr| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType) != ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType)
        }
    }

    fn get_compare_greater_than(&self) -> ScalarCompareFnImmediate {
        |current_value_ptr, immediate_value_ptr| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType) > ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType)
        }
    }

    fn get_compare_greater_than_or_equal(&self) -> ScalarCompareFnImmediate {
        |current_value_ptr, immediate_value_ptr| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType) >= ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType)
        }
    }

    fn get_compare_less_than(&self) -> ScalarCompareFnImmediate {
        |current_value_ptr, immediate_value_ptr| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType) < ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType)
        }
    }

    fn get_compare_less_than_or_equal(&self) -> ScalarCompareFnImmediate {
        |current_value_ptr, immediate_value_ptr| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType) <= ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType)
        }
    }

    fn get_compare_changed(&self) -> ScalarCompareFnRelative {
        |current_value_ptr, previous_value_ptr| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType) != ptr::read_unaligned(previous_value_ptr as *const PrimitiveType)
        }
    }

    fn get_compare_unchanged(&self) -> ScalarCompareFnRelative {
        |current_value_ptr, previous_value_ptr| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType) == ptr::read_unaligned(previous_value_ptr as *const PrimitiveType)
        }
    }

    fn get_compare_increased(&self) -> ScalarCompareFnRelative {
        |current_value_ptr, previous_value_ptr| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType) > ptr::read_unaligned(previous_value_ptr as *const PrimitiveType)
        }
    }

    fn get_compare_decreased(&self) -> ScalarCompareFnRelative {
        |current_value_ptr, previous_value_ptr| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType) < ptr::read_unaligned(previous_value_ptr as *const PrimitiveType)
        }
    }

    fn get_compare_increased_by(&self) -> ScalarCompareFnDelta {
        |current_value_ptr, previous_value_ptr, delta| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType)
                == ptr::read_unaligned(previous_value_ptr as *const PrimitiveType).wrapping_add(ptr::read_unaligned(delta as *const PrimitiveType))
        }
    }

    fn get_compare_decreased_by(&self) -> ScalarCompareFnDelta {
        |current_value_ptr, previous_value_ptr, delta| unsafe {
            ptr::read_unaligned(current_value_ptr as *const PrimitiveType)
                == ptr::read_unaligned(previous_value_ptr as *const PrimitiveType).wrapping_sub(ptr::read_unaligned(delta as *const PrimitiveType))
        }
    }
}
