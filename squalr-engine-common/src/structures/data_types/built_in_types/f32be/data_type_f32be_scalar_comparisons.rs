use crate::structures::data_types::built_in_types::f32be::data_type_f32be::DataTypeF32be;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarComparable;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnDelta;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnImmediate;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnRelative;
use crate::structures::scanning::scan_parameters::ScanParameters;
use std::mem;
use std::ops::Add;
use std::ops::Sub;
use std::ptr;

type PrimitiveType = f32;
type SwapCompatibleType = i32;

impl ScalarComparable for DataTypeF32be {
    fn get_compare_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnImmediate {
        |current_value_ptr, immediate_value_ptr| unsafe {
            // No endian byte swap required.
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
            // No endian byte swap required.
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
            let current_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                current_value_ptr as *const SwapCompatibleType,
            )));
            let immediate_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                immediate_value_ptr as *const SwapCompatibleType,
            )));

            current_value > immediate_value
        }
    }

    fn get_compare_greater_than_or_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnImmediate {
        |current_value_ptr, immediate_value_ptr| unsafe {
            let current_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                current_value_ptr as *const SwapCompatibleType,
            )));
            let immediate_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                immediate_value_ptr as *const SwapCompatibleType,
            )));

            current_value >= immediate_value
        }
    }

    fn get_compare_less_than(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnImmediate {
        |current_value_ptr, immediate_value_ptr| unsafe {
            let current_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                current_value_ptr as *const SwapCompatibleType,
            )));
            let immediate_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                immediate_value_ptr as *const SwapCompatibleType,
            )));

            current_value < immediate_value
        }
    }

    fn get_compare_less_than_or_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnImmediate {
        |current_value_ptr, immediate_value_ptr| unsafe {
            let current_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                current_value_ptr as *const SwapCompatibleType,
            )));
            let immediate_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                immediate_value_ptr as *const SwapCompatibleType,
            )));

            current_value <= immediate_value
        }
    }

    fn get_compare_changed(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative {
        |current_value_ptr, previous_value_ptr| unsafe {
            // No endian byte swap required.
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
            // No endian byte swap required.
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
            let current_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                current_value_ptr as *const SwapCompatibleType,
            )));
            let previous_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                previous_value_ptr as *const SwapCompatibleType,
            )));

            current_value > previous_value
        }
    }

    fn get_compare_decreased(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative {
        |current_value_ptr, previous_value_ptr| unsafe {
            let current_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                current_value_ptr as *const SwapCompatibleType,
            )));
            let previous_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                previous_value_ptr as *const SwapCompatibleType,
            )));

            current_value < previous_value
        }
    }

    fn get_compare_increased_by(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnDelta {
        |current_value_ptr, previous_value_ptr, delta_ptr| unsafe {
            let current_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                current_value_ptr as *const SwapCompatibleType,
            )));
            let previous_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                previous_value_ptr as *const SwapCompatibleType,
            )));
            let delta_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                delta_ptr as *const SwapCompatibleType,
            )));

            current_value == previous_value.add(delta_value)
        }
    }

    fn get_compare_decreased_by(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnDelta {
        |current_value_ptr, previous_value_ptr, delta_ptr| unsafe {
            let current_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                current_value_ptr as *const SwapCompatibleType,
            )));
            let previous_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                previous_value_ptr as *const SwapCompatibleType,
            )));
            let delta_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                delta_ptr as *const SwapCompatibleType,
            )));

            current_value == previous_value.sub(delta_value)
        }
    }
}
