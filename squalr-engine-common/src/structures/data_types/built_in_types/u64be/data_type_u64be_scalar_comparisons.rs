use crate::structures::data_types::built_in_types::u64be::data_type_u64be::DataTypeU64be;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarComparable;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnDelta;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnImmediate;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnRelative;
use crate::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use crate::structures::scanning::scan_parameters_local::ScanParametersLocal;
use std::ptr;

type PrimitiveType = u64;

impl ScalarComparable for DataTypeU64be {
    fn get_compare_equal(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnImmediate> {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            // No endian byte swap required for immediate or current values.
            unsafe {
                let immediate_value_ptr = immediate_value.as_ptr();
                let immediate_value = ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType);

                Some(Box::new(move |current_value_ptr| {
                    let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);

                    current_value == immediate_value
                }))
            }
        } else {
            None
        }
    }

    fn get_compare_not_equal(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnImmediate> {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            // No endian byte swap required for immediate or current values.
            unsafe {
                let immediate_value_ptr = immediate_value.as_ptr();
                let immediate_value = ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType);

                Some(Box::new(move |current_value_ptr| {
                    let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);

                    current_value != immediate_value
                }))
            }
        } else {
            None
        }
    }

    fn get_compare_greater_than(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnImmediate> {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            unsafe {
                let immediate_value_ptr = immediate_value.as_ptr();
                let immediate_value = PrimitiveType::swap_bytes(ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType));

                Some(Box::new(move |current_value_ptr| {
                    let current_value = PrimitiveType::swap_bytes(ptr::read_unaligned(current_value_ptr as *const PrimitiveType));

                    current_value > immediate_value
                }))
            }
        } else {
            None
        }
    }

    fn get_compare_greater_than_or_equal(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnImmediate> {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            unsafe {
                let immediate_value_ptr = immediate_value.as_ptr();
                let immediate_value = PrimitiveType::swap_bytes(ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType));

                Some(Box::new(move |current_value_ptr| {
                    let current_value = PrimitiveType::swap_bytes(ptr::read_unaligned(current_value_ptr as *const PrimitiveType));

                    current_value >= immediate_value
                }))
            }
        } else {
            None
        }
    }

    fn get_compare_less_than(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnImmediate> {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            unsafe {
                let immediate_value_ptr = immediate_value.as_ptr();
                let immediate_value = PrimitiveType::swap_bytes(ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType));

                Some(Box::new(move |current_value_ptr| {
                    let current_value = PrimitiveType::swap_bytes(ptr::read_unaligned(current_value_ptr as *const PrimitiveType));

                    // No checks tolerance required.
                    current_value < immediate_value
                }))
            }
        } else {
            None
        }
    }

    fn get_compare_less_than_or_equal(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnImmediate> {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            unsafe {
                let immediate_value_ptr = immediate_value.as_ptr();
                let immediate_value = PrimitiveType::swap_bytes(ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType));

                Some(Box::new(move |current_value_ptr| {
                    let current_value = PrimitiveType::swap_bytes(ptr::read_unaligned(current_value_ptr as *const PrimitiveType));

                    // No checks tolerance required.
                    current_value <= immediate_value
                }))
            }
        } else {
            None
        }
    }

    fn get_compare_changed(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnRelative> {
        Some(Box::new(move |current_value_ptr, previous_value_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let previous_value = ptr::read_unaligned(previous_value_ptr as *const PrimitiveType);

            current_value != previous_value
        }))
    }

    fn get_compare_unchanged(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnRelative> {
        Some(Box::new(move |current_value_ptr, previous_value_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let previous_value = ptr::read_unaligned(previous_value_ptr as *const PrimitiveType);

            current_value == previous_value
        }))
    }

    fn get_compare_increased(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnRelative> {
        Some(Box::new(move |current_value_ptr, previous_value_ptr| unsafe {
            let current_value = PrimitiveType::swap_bytes(ptr::read_unaligned(current_value_ptr as *const PrimitiveType));
            let previous_value = PrimitiveType::swap_bytes(ptr::read_unaligned(previous_value_ptr as *const PrimitiveType));

            current_value > previous_value
        }))
    }

    fn get_compare_decreased(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnRelative> {
        Some(Box::new(move |current_value_ptr, previous_value_ptr| unsafe {
            let current_value = PrimitiveType::swap_bytes(ptr::read_unaligned(current_value_ptr as *const PrimitiveType));
            let previous_value = PrimitiveType::swap_bytes(ptr::read_unaligned(previous_value_ptr as *const PrimitiveType));

            current_value < previous_value
        }))
    }

    fn get_compare_increased_by(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnDelta> {
        if let Some(delta_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            unsafe {
                let delta_value_ptr = delta_value.as_ptr();
                let delta_value = PrimitiveType::swap_bytes(ptr::read_unaligned(delta_value_ptr as *const PrimitiveType));

                Some(Box::new(move |current_value_ptr, previous_value_ptr| {
                    let current_value = PrimitiveType::swap_bytes(ptr::read_unaligned(current_value_ptr as *const PrimitiveType));
                    let previous_value = PrimitiveType::swap_bytes(ptr::read_unaligned(previous_value_ptr as *const PrimitiveType));
                    let target_value = previous_value.wrapping_add(delta_value);

                    current_value == target_value
                }))
            }
        } else {
            None
        }
    }

    fn get_compare_decreased_by(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnDelta> {
        if let Some(delta_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            unsafe {
                let delta_value_ptr = delta_value.as_ptr();
                let delta_value = PrimitiveType::swap_bytes(ptr::read_unaligned(delta_value_ptr as *const PrimitiveType));

                Some(Box::new(move |current_value_ptr, previous_value_ptr| {
                    let current_value = PrimitiveType::swap_bytes(ptr::read_unaligned(current_value_ptr as *const PrimitiveType));
                    let previous_value = PrimitiveType::swap_bytes(ptr::read_unaligned(previous_value_ptr as *const PrimitiveType));
                    let target_value = previous_value.wrapping_sub(delta_value);

                    current_value == target_value
                }))
            }
        } else {
            None
        }
    }
}
