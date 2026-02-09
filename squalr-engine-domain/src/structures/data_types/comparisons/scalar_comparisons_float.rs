use crate::structures::scanning::comparisons::scan_function_scalar::{ScalarCompareFnDelta, ScalarCompareFnImmediate, ScalarCompareFnRelative};
use crate::structures::scanning::constraints::scan_constraint::ScanConstraint;
use num_traits::Float;
use std::ops::{Add, Sub};
use std::ptr;
use std::sync::Arc;

pub struct ScalarComparisonsFloat {}

impl ScalarComparisonsFloat {
    pub fn get_compare_equal<PrimitiveType: PartialEq + Float + Sub<Output = PrimitiveType> + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = scan_constraint.get_data_value();
        let tolerance = scan_constraint.get_floating_point_tolerance().get_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) };

        Some(Arc::new(move |current_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };

            // Equality between the current and immediate value is determined by being within the given tolerance.
            current_value.sub(immediate_value).abs() <= tolerance
        }))
    }

    pub fn get_compare_not_equal<PrimitiveType: PartialEq + Float + Sub<Output = PrimitiveType> + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = scan_constraint.get_data_value();
        let tolerance = scan_constraint.get_floating_point_tolerance().get_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) };

        Some(Arc::new(move |current_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };

            // Inequality between the current and immediate value is determined by being outside the given tolerance.
            current_value.sub(immediate_value).abs() > tolerance
        }))
    }

    pub fn get_compare_greater_than<PrimitiveType: PartialOrd + Send + Sync + 'static>(scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = scan_constraint.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) };

        Some(Arc::new(move |current_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };

            // No checks tolerance required.
            current_value > immediate_value
        }))
    }

    pub fn get_compare_greater_than_or_equal<PrimitiveType: PartialOrd + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = scan_constraint.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) };

        Some(Arc::new(move |current_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };

            // No checks tolerance required.
            current_value >= immediate_value
        }))
    }

    pub fn get_compare_less_than<PrimitiveType: PartialOrd + Send + Sync + 'static>(scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = scan_constraint.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) };

        Some(Arc::new(move |current_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };

            // No checks tolerance required.
            current_value < immediate_value
        }))
    }

    pub fn get_compare_less_than_or_equal<PrimitiveType: PartialOrd + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = scan_constraint.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) };

        Some(Arc::new(move |current_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };

            // No checks tolerance required.
            current_value <= immediate_value
        }))
    }

    pub fn get_compare_changed<PrimitiveType: PartialEq + Send + Sync + 'static>(_scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnRelative> {
        Some(Arc::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };
            let previous_value = unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) };

            // No checks tolerance required.
            current_value != previous_value
        }))
    }

    pub fn get_compare_unchanged<PrimitiveType: PartialEq + Send + Sync + 'static>(_scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnRelative> {
        Some(Arc::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };
            let previous_value = unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) };

            // No checks tolerance required.
            current_value == previous_value
        }))
    }

    pub fn get_compare_increased<PrimitiveType: PartialOrd + Send + Sync + 'static>(_scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnRelative> {
        Some(Arc::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };
            let previous_value = unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) };

            // No checks tolerance required.
            current_value > previous_value
        }))
    }

    pub fn get_compare_decreased<PrimitiveType: PartialOrd + Send + Sync + 'static>(_scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnRelative> {
        Some(Arc::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };
            let previous_value = unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) };

            // No checks tolerance required.
            current_value < previous_value
        }))
    }

    pub fn get_compare_increased_by<
        PrimitiveType: Copy + PartialEq + Float + Add<Output = PrimitiveType> + Sub<Output = PrimitiveType> + Send + Sync + 'static,
    >(
        scan_constraint: &ScanConstraint
    ) -> Option<ScalarCompareFnDelta> {
        let immediate_value = scan_constraint.get_data_value();
        let tolerance = scan_constraint.get_floating_point_tolerance().get_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) };

        Some(Arc::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };
            let previous_value = unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) };
            let target_value = previous_value.add(delta_value);

            // Equality between the current and target value is determined by being within the given tolerance.
            current_value.sub(target_value).abs() <= tolerance
        }))
    }

    pub fn get_compare_decreased_by<
        PrimitiveType: Copy + PartialEq + Float + Add<Output = PrimitiveType> + Sub<Output = PrimitiveType> + Send + Sync + 'static,
    >(
        scan_constraint: &ScanConstraint
    ) -> Option<ScalarCompareFnDelta> {
        let immediate_value = scan_constraint.get_data_value();
        let tolerance = scan_constraint.get_floating_point_tolerance().get_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) };

        Some(Arc::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };
            let previous_value = unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) };
            let target_value = previous_value.sub(delta_value);

            // Equality between the current and target value is determined by being within the given tolerance.
            current_value.sub(target_value).abs() <= tolerance
        }))
    }

    pub fn get_compare_multiplied_by<
        PrimitiveType: Copy + PartialEq + Float + Add<Output = PrimitiveType> + Sub<Output = PrimitiveType> + Send + Sync + 'static,
    >(
        scan_constraint: &ScanConstraint
    ) -> Option<ScalarCompareFnDelta> {
        let immediate_value = scan_constraint.get_data_value();
        let tolerance = scan_constraint.get_floating_point_tolerance().get_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) };

        Some(Arc::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };
            let previous_value = unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) };
            let target_value = previous_value.mul(delta_value);

            // Equality between the current and target value is determined by being within the given tolerance.
            current_value.sub(target_value).abs() <= tolerance
        }))
    }

    pub fn get_compare_divided_by<
        PrimitiveType: Copy + PartialEq + Float + Add<Output = PrimitiveType> + Sub<Output = PrimitiveType> + Send + Sync + 'static,
    >(
        scan_constraint: &ScanConstraint
    ) -> Option<ScalarCompareFnDelta> {
        let immediate_value = scan_constraint.get_data_value();
        let tolerance = scan_constraint.get_floating_point_tolerance().get_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) };

        Some(Arc::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };
            let previous_value = unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) };
            let target_value = previous_value.div(delta_value);

            // Equality between the current and target value is determined by being within the given tolerance.
            current_value.sub(target_value).abs() <= tolerance
        }))
    }

    pub fn get_compare_modulo_by<
        PrimitiveType: Copy + PartialEq + Float + Add<Output = PrimitiveType> + Sub<Output = PrimitiveType> + Send + Sync + 'static,
    >(
        scan_constraint: &ScanConstraint
    ) -> Option<ScalarCompareFnDelta> {
        let immediate_value = scan_constraint.get_data_value();
        let tolerance = scan_constraint.get_floating_point_tolerance().get_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) };

        Some(Arc::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };
            let previous_value = unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) };
            let target_value = previous_value.rem(delta_value);

            // Equality between the current and target value is determined by being within the given tolerance.
            current_value.sub(target_value).abs() <= tolerance
        }))
    }
}
