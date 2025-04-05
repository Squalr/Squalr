use crate::structures::data_types::comparisons::scalar_comparable::{ScalarCompareFnDelta, ScalarCompareFnImmediate, ScalarCompareFnRelative};
use crate::structures::scanning::parameters::mapped_scan_parameters::ScanParametersCommon;
use num_traits::Float;
use std::ops::{Add, Sub};
use std::ptr;

pub struct ScalarComparisonsFloat {}

impl ScalarComparisonsFloat {
    pub fn get_compare_equal<PrimitiveType: PartialEq + Float + Sub<Output = PrimitiveType> + 'static>(
        scan_parameters: &ScanParametersCommon
    ) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = scan_parameters.get_data_value();
        let tolerance = scan_parameters.get_floating_point_tolerance().get_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) };

        Some(Box::new(move |current_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };

            // Equality between the current and immediate value is determined by being within the given tolerance.
            current_value.sub(immediate_value).abs() <= tolerance
        }))
    }

    pub fn get_compare_not_equal<PrimitiveType: PartialEq + Float + Sub<Output = PrimitiveType> + 'static>(
        scan_parameters: &ScanParametersCommon
    ) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = scan_parameters.get_data_value();
        let tolerance = scan_parameters.get_floating_point_tolerance().get_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) };

        Some(Box::new(move |current_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };

            // Inequality between the current and immediate value is determined by being outside the given tolerance.
            current_value.sub(immediate_value).abs() > tolerance
        }))
    }

    pub fn get_compare_greater_than<PrimitiveType: PartialOrd + 'static>(scan_parameters: &ScanParametersCommon) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) };

        Some(Box::new(move |current_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };

            // No checks tolerance required.
            current_value > immediate_value
        }))
    }

    pub fn get_compare_greater_than_or_equal<PrimitiveType: PartialOrd + 'static>(scan_parameters: &ScanParametersCommon) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) };

        Some(Box::new(move |current_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };

            // No checks tolerance required.
            current_value >= immediate_value
        }))
    }

    pub fn get_compare_less_than<PrimitiveType: PartialOrd + 'static>(scan_parameters: &ScanParametersCommon) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) };

        Some(Box::new(move |current_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };

            // No checks tolerance required.
            current_value < immediate_value
        }))
    }

    pub fn get_compare_less_than_or_equal<PrimitiveType: PartialOrd + 'static>(scan_parameters: &ScanParametersCommon) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) };

        Some(Box::new(move |current_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };

            // No checks tolerance required.
            current_value <= immediate_value
        }))
    }

    pub fn get_compare_changed<PrimitiveType: PartialEq + 'static>(_scan_parameters: &ScanParametersCommon) -> Option<ScalarCompareFnRelative> {
        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };
            let previous_value = unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) };

            // No checks tolerance required.
            current_value != previous_value
        }))
    }

    pub fn get_compare_unchanged<PrimitiveType: PartialEq + 'static>(_scan_parameters: &ScanParametersCommon) -> Option<ScalarCompareFnRelative> {
        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };
            let previous_value = unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) };

            // No checks tolerance required.
            current_value == previous_value
        }))
    }

    pub fn get_compare_increased<PrimitiveType: PartialOrd + 'static>(_scan_parameters: &ScanParametersCommon) -> Option<ScalarCompareFnRelative> {
        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };
            let previous_value = unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) };

            // No checks tolerance required.
            current_value > previous_value
        }))
    }

    pub fn get_compare_decreased<PrimitiveType: PartialOrd + 'static>(_scan_parameters: &ScanParametersCommon) -> Option<ScalarCompareFnRelative> {
        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };
            let previous_value = unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) };

            // No checks tolerance required.
            current_value < previous_value
        }))
    }

    pub fn get_compare_increased_by<PrimitiveType: Copy + PartialEq + Float + Add<Output = PrimitiveType> + Sub<Output = PrimitiveType> + 'static>(
        scan_parameters: &ScanParametersCommon
    ) -> Option<ScalarCompareFnDelta> {
        let immediate_value = scan_parameters.get_data_value();
        let tolerance = scan_parameters.get_floating_point_tolerance().get_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) };

        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };
            let previous_value = unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) };
            let target_value = previous_value.add(delta_value);

            // Equality between the current and target value is determined by being within the given tolerance.
            current_value.sub(target_value).abs() <= tolerance
        }))
    }

    pub fn get_compare_decreased_by<PrimitiveType: Copy + PartialEq + Float + Add<Output = PrimitiveType> + Sub<Output = PrimitiveType> + 'static>(
        scan_parameters: &ScanParametersCommon
    ) -> Option<ScalarCompareFnDelta> {
        let immediate_value = scan_parameters.get_data_value();
        let tolerance = scan_parameters.get_floating_point_tolerance().get_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) };

        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };
            let previous_value = unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) };
            let target_value = previous_value.sub(delta_value);

            // Equality between the current and target value is determined by being within the given tolerance.
            current_value.sub(target_value).abs() <= tolerance
        }))
    }
}
