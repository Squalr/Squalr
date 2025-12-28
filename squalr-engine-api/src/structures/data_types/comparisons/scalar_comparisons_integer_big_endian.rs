use crate::structures::scanning::comparisons::scan_function_scalar::{ScalarCompareFnDelta, ScalarCompareFnImmediate, ScalarCompareFnRelative};
use crate::structures::scanning::plans::element_scan::snapshot_filter_element_scan_plan::SnapshotFilterElementScanPlan;
use num_traits::{PrimInt, WrappingAdd, WrappingMul, WrappingSub};
use std::ops::{BitAnd, BitOr, BitXor, Div, Rem, Shl, Shr};
use std::ptr;

pub struct ScalarComparisonsIntegerBigEndian {}

impl ScalarComparisonsIntegerBigEndian {
    pub fn get_compare_equal<PrimitiveType: PartialEq + 'static>(scan_parameters: &SnapshotFilterElementScanPlan) -> Option<ScalarCompareFnImmediate> {
        // Optimization: no endian byte swap required for immediate or current values.
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) };

        Some(Box::new(move |current_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };

            current_value == immediate_value
        }))
    }

    pub fn get_compare_not_equal<PrimitiveType: PartialEq + 'static>(scan_parameters: &SnapshotFilterElementScanPlan) -> Option<ScalarCompareFnImmediate> {
        // Optimization: no endian byte swap required for immediate or current values.
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) };

        Some(Box::new(move |current_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };

            current_value != immediate_value
        }))
    }

    pub fn get_compare_greater_than<PrimitiveType: PartialOrd + PrimInt + 'static>(
        scan_parameters: &SnapshotFilterElementScanPlan
    ) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) });

        Some(Box::new(move |current_value_ptr| {
            let current_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) });

            current_value > immediate_value
        }))
    }

    pub fn get_compare_greater_than_or_equal<PrimitiveType: PartialOrd + PrimInt + 'static>(
        scan_parameters: &SnapshotFilterElementScanPlan
    ) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) });

        Some(Box::new(move |current_value_ptr| {
            let current_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) });

            current_value >= immediate_value
        }))
    }

    pub fn get_compare_less_than<PrimitiveType: PartialOrd + PrimInt + 'static>(
        scan_parameters: &SnapshotFilterElementScanPlan
    ) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) });

        Some(Box::new(move |current_value_ptr| {
            let current_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) });

            // No checks tolerance required.
            current_value < immediate_value
        }))
    }

    pub fn get_compare_less_than_or_equal<PrimitiveType: PartialOrd + PrimInt + 'static>(
        scan_parameters: &SnapshotFilterElementScanPlan
    ) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) });

        Some(Box::new(move |current_value_ptr| {
            let current_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) });

            // No checks tolerance required.
            current_value <= immediate_value
        }))
    }

    pub fn get_compare_changed<PrimitiveType: PartialEq + 'static>(_scan_parameters: &SnapshotFilterElementScanPlan) -> Option<ScalarCompareFnRelative> {
        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };
            let previous_value = unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) };

            current_value != previous_value
        }))
    }

    pub fn get_compare_unchanged<PrimitiveType: PartialEq + 'static>(_scan_parameters: &SnapshotFilterElementScanPlan) -> Option<ScalarCompareFnRelative> {
        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };
            let previous_value = unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) };

            current_value == previous_value
        }))
    }

    pub fn get_compare_increased<PrimitiveType: PartialOrd + PrimInt + 'static>(
        _scan_parameters: &SnapshotFilterElementScanPlan
    ) -> Option<ScalarCompareFnRelative> {
        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) });
            let previous_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) });

            current_value > previous_value
        }))
    }

    pub fn get_compare_decreased<PrimitiveType: PartialOrd + PrimInt + 'static>(
        _scan_parameters: &SnapshotFilterElementScanPlan
    ) -> Option<ScalarCompareFnRelative> {
        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) });
            let previous_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) });

            current_value < previous_value
        }))
    }

    pub fn get_compare_increased_by<PrimitiveType: Copy + PartialEq + PrimInt + WrappingAdd<Output = PrimitiveType> + WrappingAdd + 'static>(
        scan_parameters: &SnapshotFilterElementScanPlan
    ) -> Option<ScalarCompareFnDelta> {
        let immediate_value = scan_parameters.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value: PrimitiveType = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) });

        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) });
            let previous_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) });
            let target_value = previous_value.wrapping_add(&delta_value);

            current_value == target_value
        }))
    }

    pub fn get_compare_decreased_by<PrimitiveType: Copy + PartialEq + PrimInt + WrappingSub<Output = PrimitiveType> + 'static>(
        scan_parameters: &SnapshotFilterElementScanPlan
    ) -> Option<ScalarCompareFnDelta> {
        let immediate_value = scan_parameters.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value: PrimitiveType = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) });

        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) });
            let previous_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) });
            let target_value = previous_value.wrapping_sub(&delta_value);

            current_value == target_value
        }))
    }

    pub fn get_compare_multiplied_by<PrimitiveType: Copy + PartialEq + PrimInt + WrappingMul<Output = PrimitiveType> + 'static>(
        scan_parameters: &SnapshotFilterElementScanPlan
    ) -> Option<ScalarCompareFnDelta> {
        let immediate_value = scan_parameters.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value: PrimitiveType = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) });

        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) });
            let previous_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) });
            let target_value = previous_value.wrapping_mul(&delta_value);

            current_value == target_value
        }))
    }

    pub fn get_compare_divided_by<PrimitiveType: Copy + PartialEq + PrimInt + Div<Output = PrimitiveType> + Default + 'static>(
        scan_parameters: &SnapshotFilterElementScanPlan
    ) -> Option<ScalarCompareFnDelta> {
        let immediate_value = scan_parameters.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value: PrimitiveType = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) });

        // Disallow divide by zero.
        if delta_value == PrimitiveType::default() {
            return None;
        }

        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) });
            let previous_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) });
            let target_value = previous_value.div(delta_value);

            current_value == target_value
        }))
    }

    pub fn get_compare_modulo_by<PrimitiveType: Copy + PartialEq + PrimInt + Rem<Output = PrimitiveType> + Default + 'static>(
        scan_parameters: &SnapshotFilterElementScanPlan
    ) -> Option<ScalarCompareFnDelta> {
        let immediate_value = scan_parameters.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value: PrimitiveType = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) });

        // Disallow divide by zero.
        if delta_value == PrimitiveType::default() {
            return None;
        }

        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) });
            let previous_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) });
            let target_value = previous_value.rem(delta_value);

            current_value == target_value
        }))
    }

    pub fn get_compare_shift_left_by<PrimitiveType: Copy + PartialEq + PrimInt + Shl<Output = PrimitiveType> + 'static>(
        scan_parameters: &SnapshotFilterElementScanPlan
    ) -> Option<ScalarCompareFnDelta> {
        let immediate_value = scan_parameters.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value: PrimitiveType = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) });

        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) });
            let previous_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) });
            let target_value = previous_value.shl(delta_value);

            current_value == target_value
        }))
    }

    pub fn get_compare_shift_right_by<PrimitiveType: Copy + PartialEq + PrimInt + Shr<Output = PrimitiveType> + 'static>(
        scan_parameters: &SnapshotFilterElementScanPlan
    ) -> Option<ScalarCompareFnDelta> {
        let immediate_value = scan_parameters.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value: PrimitiveType = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) });

        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) });
            let previous_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) });
            let target_value = previous_value.shr(delta_value);

            current_value == target_value
        }))
    }

    pub fn get_compare_logical_and_by<PrimitiveType: Copy + PartialEq + PrimInt + BitAnd<Output = PrimitiveType> + 'static>(
        scan_parameters: &SnapshotFilterElementScanPlan
    ) -> Option<ScalarCompareFnDelta> {
        let immediate_value = scan_parameters.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value: PrimitiveType = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) });

        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) });
            let previous_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) });
            let target_value = previous_value.bitand(delta_value);

            current_value == target_value
        }))
    }

    pub fn get_compare_logical_or_by<PrimitiveType: Copy + PartialEq + PrimInt + BitOr<Output = PrimitiveType> + 'static>(
        scan_parameters: &SnapshotFilterElementScanPlan
    ) -> Option<ScalarCompareFnDelta> {
        let immediate_value = scan_parameters.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value: PrimitiveType = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) });

        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) });
            let previous_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) });
            let target_value = previous_value.bitor(delta_value);

            current_value == target_value
        }))
    }

    pub fn get_compare_logical_xor_by<PrimitiveType: Copy + PartialEq + PrimInt + BitXor<Output = PrimitiveType> + 'static>(
        scan_parameters: &SnapshotFilterElementScanPlan
    ) -> Option<ScalarCompareFnDelta> {
        let immediate_value = scan_parameters.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value: PrimitiveType = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) });

        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) });
            let previous_value = PrimitiveType::swap_bytes(unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) });
            let target_value = previous_value.bitxor(delta_value);

            current_value == target_value
        }))
    }
}
