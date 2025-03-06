use crate::structures::data_types::built_in_types::f32be::data_type_f32be::DataTypeF32be;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarComparable;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnDelta;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnImmediate;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnRelative;
use crate::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use crate::structures::scanning::scan_parameters_local::ScanParametersLocal;
use std::mem;
use std::ops::Add;
use std::ops::Sub;
use std::ptr;

type PrimitiveType = f32;
type SwapCompatibleType = i32;

impl ScalarComparable for DataTypeF32be {
    fn get_compare_equal(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnImmediate> {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            // No endian byte swap required for immediate or current values.
            unsafe {
                let tolerance = scan_parameters_global
                    .get_floating_point_tolerance()
                    .get_value_f32();
                let immediate_value_ptr = immediate_value.as_ptr();
                let immediate_value = ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType);

                Some(Box::new(move |current_value_ptr| {
                    let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);

                    // Equality between the current and immediate value is determined by being within the given tolerance.
                    current_value.sub(immediate_value).abs() <= tolerance
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
                let tolerance = scan_parameters_global
                    .get_floating_point_tolerance()
                    .get_value_f32();
                let immediate_value_ptr = immediate_value.as_ptr();
                let immediate_value = ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType);

                Some(Box::new(move |current_value_ptr| {
                    let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);

                    // Inequality between the current and immediate value is determined by being outside the given tolerance.
                    current_value.sub(immediate_value).abs() > tolerance
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
                let immediate_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                    immediate_value_ptr as *const SwapCompatibleType,
                )));

                Some(Box::new(move |current_value_ptr| {
                    let current_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                        current_value_ptr as *const SwapCompatibleType,
                    )));

                    // No checks tolerance required.
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
                let immediate_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                    immediate_value_ptr as *const SwapCompatibleType,
                )));

                Some(Box::new(move |current_value_ptr| {
                    let current_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                        current_value_ptr as *const SwapCompatibleType,
                    )));

                    // No checks tolerance required.
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
                let immediate_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                    immediate_value_ptr as *const SwapCompatibleType,
                )));

                Some(Box::new(move |current_value_ptr| {
                    let current_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                        current_value_ptr as *const SwapCompatibleType,
                    )));

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
                let immediate_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                    immediate_value_ptr as *const SwapCompatibleType,
                )));

                Some(Box::new(move |current_value_ptr| {
                    let current_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                        current_value_ptr as *const SwapCompatibleType,
                    )));

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
            // No endian byte swaps required for current or previous values.
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let previous_value = ptr::read_unaligned(previous_value_ptr as *const PrimitiveType);

            // No checks tolerance required.
            current_value != previous_value
        }))
    }

    fn get_compare_unchanged(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnRelative> {
        Some(Box::new(move |current_value_ptr, previous_value_ptr| unsafe {
            // No endian byte swaps required for current or previous values.
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let previous_value = ptr::read_unaligned(previous_value_ptr as *const PrimitiveType);

            // No checks tolerance required.
            current_value == previous_value
        }))
    }

    fn get_compare_increased(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnRelative> {
        Some(Box::new(move |current_value_ptr, previous_value_ptr| unsafe {
            let current_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                current_value_ptr as *const SwapCompatibleType,
            )));
            let previous_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                previous_value_ptr as *const SwapCompatibleType,
            )));

            // No checks tolerance required.
            current_value > previous_value
        }))
    }

    fn get_compare_decreased(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnRelative> {
        Some(Box::new(move |current_value_ptr, previous_value_ptr| unsafe {
            let current_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                current_value_ptr as *const SwapCompatibleType,
            )));
            let previous_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                previous_value_ptr as *const SwapCompatibleType,
            )));

            // No checks tolerance required.
            current_value < previous_value
        }))
    }

    fn get_compare_increased_by(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnDelta> {
        if let Some(delta_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            // No endian byte swap required for immediate or current values.
            unsafe {
                let tolerance = scan_parameters_global
                    .get_floating_point_tolerance()
                    .get_value_f32();
                let delta_value_ptr = delta_value.as_ptr();
                let delta_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                    delta_value_ptr as *const SwapCompatibleType,
                )));

                Some(Box::new(move |current_value_ptr, previous_value_ptr| {
                    let current_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                        current_value_ptr as *const SwapCompatibleType,
                    )));
                    let previous_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                        previous_value_ptr as *const SwapCompatibleType,
                    )));
                    let target_value = previous_value.add(delta_value);

                    // Equality between the current and target value is determined by being within the given tolerance.
                    current_value.sub(target_value).abs() <= tolerance
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
            // No endian byte swap required for immediate or current values.
            unsafe {
                let tolerance = scan_parameters_global
                    .get_floating_point_tolerance()
                    .get_value_f32();
                let delta_value_ptr = delta_value.as_ptr();
                let delta_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                    delta_value_ptr as *const SwapCompatibleType,
                )));

                Some(Box::new(move |current_value_ptr, previous_value_ptr| {
                    let current_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                        current_value_ptr as *const SwapCompatibleType,
                    )));
                    let previous_value = mem::transmute::<SwapCompatibleType, PrimitiveType>(SwapCompatibleType::swap_bytes(ptr::read_unaligned(
                        previous_value_ptr as *const SwapCompatibleType,
                    )));
                    let target_value = previous_value.sub(delta_value);

                    // Equality between the current and target value is determined by being within the given tolerance.
                    current_value.sub(target_value).abs() <= tolerance
                }))
            }
        } else {
            None
        }
    }
}
