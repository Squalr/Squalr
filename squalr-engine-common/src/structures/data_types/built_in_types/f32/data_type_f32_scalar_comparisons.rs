use crate::structures::data_types::built_in_types::f32::data_type_f32::DataTypeF32;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarComparable;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnDelta;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnImmediate;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnRelative;
use crate::structures::scanning::scan_parameters::ScanParameters;
use std::ops::Add;
use std::ops::Sub;
use std::ptr;

type PrimitiveType = f32;

impl ScalarComparable for DataTypeF32 {
    fn get_compare_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnImmediate {
        let tolerance = scan_parameters.get_floating_point_tolerance().get_value_f32();

        Box::new(move |current_value_ptr, immediate_value_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let immediate_value = ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType);

            // Equality between the current and immediate value is determined by being within the given tolerance.
            current_value.sub(immediate_value).abs() <= tolerance
        })
    }

    fn get_compare_not_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnImmediate {
        let tolerance = scan_parameters.get_floating_point_tolerance().get_value_f32();

        Box::new(move |current_value_ptr, immediate_value_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let immediate_value = ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType);

            // Inequality between the current and immediate value is determined by being outside the given tolerance.
            current_value.sub(immediate_value).abs() > tolerance
        })
    }

    fn get_compare_greater_than(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnImmediate {
        Box::new(move |current_value_ptr, immediate_value_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let immediate_value = ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType);

            // No checks tolerance required.
            current_value > immediate_value
        })
    }

    fn get_compare_greater_than_or_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnImmediate {
        Box::new(move |current_value_ptr, immediate_value_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let immediate_value = ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType);

            // No checks tolerance required.
            current_value >= immediate_value
        })
    }

    fn get_compare_less_than(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnImmediate {
        Box::new(move |current_value_ptr, immediate_value_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let immediate_value = ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType);

            // No checks tolerance required.
            current_value < immediate_value
        })
    }

    fn get_compare_less_than_or_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnImmediate {
        Box::new(move |current_value_ptr, immediate_value_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let immediate_value = ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType);

            // No checks tolerance required.
            current_value <= immediate_value
        })
    }

    fn get_compare_changed(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative {
        Box::new(move |current_value_ptr, previous_value_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let previous_value = ptr::read_unaligned(previous_value_ptr as *const PrimitiveType);

            // No checks tolerance required.
            current_value != previous_value
        })
    }

    fn get_compare_unchanged(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative {
        Box::new(move |current_value_ptr, previous_value_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let previous_value = ptr::read_unaligned(previous_value_ptr as *const PrimitiveType);

            // No checks tolerance required.
            current_value == previous_value
        })
    }

    fn get_compare_increased(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative {
        Box::new(move |current_value_ptr, previous_value_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let previous_value = ptr::read_unaligned(previous_value_ptr as *const PrimitiveType);

            // No checks tolerance required.
            current_value > previous_value
        })
    }

    fn get_compare_decreased(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative {
        Box::new(move |current_value_ptr, previous_value_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let previous_value = ptr::read_unaligned(previous_value_ptr as *const PrimitiveType);

            // No checks tolerance required.
            current_value < previous_value
        })
    }

    fn get_compare_increased_by(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnDelta {
        let tolerance = scan_parameters.get_floating_point_tolerance().get_value_f32();

        Box::new(move |current_value_ptr, previous_value_ptr, delta_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let previous_value = ptr::read_unaligned(previous_value_ptr as *const PrimitiveType);
            let delta_value = ptr::read_unaligned(delta_ptr as *const PrimitiveType);
            let expected_value = previous_value.add(delta_value);

            // Equality between the current and expected value is determined by being within the given tolerance.
            current_value.sub(expected_value).abs() <= tolerance
        })
    }

    fn get_compare_decreased_by(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnDelta {
        let tolerance = scan_parameters.get_floating_point_tolerance().get_value_f32();

        Box::new(move |current_value_ptr, previous_value_ptr, delta_ptr| unsafe {
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let previous_value = ptr::read_unaligned(previous_value_ptr as *const PrimitiveType);
            let delta_value = ptr::read_unaligned(delta_ptr as *const PrimitiveType);
            let expected_value = previous_value.sub(delta_value);

            // Equality between the current and expected value is determined by being within the given tolerance.
            current_value.sub(expected_value).abs() <= tolerance
        })
    }
}
