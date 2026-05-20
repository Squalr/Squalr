use crate::structures::scanning::comparisons::scan_function_scalar::{ScalarCompareFnDelta, ScalarCompareFnImmediate, ScalarCompareFnRelative};
use crate::structures::scanning::constraints::scan_constraint::ScanConstraint;
use num_traits::{WrappingAdd, WrappingSub};
use std::cmp::Ordering;
use std::ops::{BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr};
use std::sync::Arc;

/// Scalar comparison functions for comparing byte arrays. Note that these functions operate on single array values.
/// For performance-critical scans, specialized algorithms are implemented elsewhere.
/// Additionally, many comparison functions for arrays are not defined, such as inequalities or delta scans,
/// and are subject to change. The only clearly defined behaviors are: equal, not equal, changed, and unchanged.
pub struct ScalarComparisonsByteArray {}

impl ScalarComparisonsByteArray {
    pub fn get_compare_equal(scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnImmediate> {
        let immediate_values = scan_constraint.get_data_value();
        let immediate_values = immediate_values.get_value_bytes().clone();
        let len = immediate_values.len();

        Some(Arc::new(move |current_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);
            current_values == immediate_values
        }))
    }

    pub fn get_compare_not_equal(scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnImmediate> {
        let immediate_values = scan_constraint.get_data_value();
        let immediate_values = immediate_values.get_value_bytes().clone();
        let len = immediate_values.len();

        Some(Arc::new(move |current_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);
            current_values != immediate_values
        }))
    }

    pub fn get_compare_greater_than(scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnImmediate> {
        let immediate_values = scan_constraint.get_data_value();
        let immediate_values = immediate_values.get_value_bytes().clone();
        let len = immediate_values.len();

        Some(Arc::new(move |current_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);
            current_values
                .iter()
                .zip(immediate_values.iter())
                .all(|(current_values, immediate_values)| current_values > immediate_values)
        }))
    }

    pub fn get_compare_greater_than_or_equal(scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnImmediate> {
        let immediate_values = scan_constraint.get_data_value();
        let immediate_values = immediate_values.get_value_bytes().clone();
        let len = immediate_values.len();

        Some(Arc::new(move |current_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);

            current_values.cmp(&immediate_values) == Ordering::Greater || current_values == immediate_values
        }))
    }

    pub fn get_compare_less_than(scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnImmediate> {
        let immediate_values = scan_constraint.get_data_value();
        let immediate_values = immediate_values.get_value_bytes().clone();
        let len = immediate_values.len();

        Some(Arc::new(move |current_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);

            current_values.cmp(&immediate_values) == Ordering::Less
        }))
    }

    pub fn get_compare_less_than_or_equal(scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnImmediate> {
        let immediate_values = scan_constraint.get_data_value();
        let immediate_values = immediate_values.get_value_bytes().clone();
        let len = immediate_values.len();

        Some(Arc::new(move |current_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);

            current_values.cmp(&immediate_values) == Ordering::Less || current_values == immediate_values
        }))
    }

    pub fn get_compare_changed(scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnRelative> {
        let len = scan_constraint.get_data_value().get_size_in_bytes() as usize;

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);
            let previous_values = std::slice::from_raw_parts(previous_values_ptr, len);
            current_values != previous_values
        }))
    }

    pub fn get_compare_unchanged(scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnRelative> {
        let len = scan_constraint.get_data_value().get_size_in_bytes() as usize;

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);
            let previous_values = std::slice::from_raw_parts(previous_values_ptr, len);

            current_values == previous_values
        }))
    }

    pub fn get_compare_increased(scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnRelative> {
        let len = scan_constraint.get_data_value().get_size_in_bytes() as usize;

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);
            let previous_values = std::slice::from_raw_parts(previous_values_ptr, len);

            current_values
                .iter()
                .zip(previous_values.iter())
                .all(|(current_value, previous_value)| current_value > previous_value)
        }))
    }

    pub fn get_compare_decreased(scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnRelative> {
        let len = scan_constraint.get_data_value().get_size_in_bytes() as usize;

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);
            let previous_values = std::slice::from_raw_parts(previous_values_ptr, len);

            current_values
                .iter()
                .zip(previous_values.iter())
                .all(|(current_value, previous_value)| current_value < previous_value)
        }))
    }

    pub fn get_compare_increased_by(scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnDelta> {
        let immediate_values = scan_constraint.get_data_value();
        let delta_values = immediate_values.get_value_bytes().clone();
        let len = delta_values.len();

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);
            let previous_values = std::slice::from_raw_parts(previous_values_ptr, len);

            current_values
                .iter()
                .zip(previous_values.iter())
                .zip(delta_values.iter())
                .all(|((current_value, previous_value), delta_value)| current_value.wrapping_add(delta_value) == *previous_value)
        }))
    }

    pub fn get_compare_decreased_by(scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnDelta> {
        let immediate_values = scan_constraint.get_data_value();
        let delta_values = immediate_values.get_value_bytes().clone();
        let len = delta_values.len();

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);
            let previous_values = std::slice::from_raw_parts(previous_values_ptr, len);

            current_values
                .iter()
                .zip(previous_values.iter())
                .zip(delta_values.iter())
                .all(|((current_value, previous_value), delta_value)| current_value.wrapping_sub(delta_value) == *previous_value)
        }))
    }

    pub fn get_compare_multiplied_by(scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnDelta> {
        let immediate_values = scan_constraint.get_data_value();
        let delta_values = immediate_values.get_value_bytes().clone();
        let len = delta_values.len();

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);
            let previous_values = std::slice::from_raw_parts(previous_values_ptr, len);

            current_values
                .iter()
                .zip(previous_values.iter())
                .zip(delta_values.iter())
                .all(|((current_value, previous_value), delta_value)| current_value.mul(*delta_value) == *previous_value)
        }))
    }

    pub fn get_compare_divided_by(scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnDelta> {
        let immediate_values = scan_constraint.get_data_value();
        let delta_values = immediate_values.get_value_bytes().clone();
        let len = delta_values.len();

        if delta_values.iter().any(|value| *value == 0) {
            return None;
        }

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);
            let previous_values = std::slice::from_raw_parts(previous_values_ptr, len);

            current_values
                .iter()
                .zip(previous_values.iter())
                .zip(delta_values.iter())
                .all(|((current_value, previous_value), delta_value)| current_value.div(*delta_value) == *previous_value)
        }))
    }

    pub fn get_compare_modulo_by(scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnDelta> {
        let immediate_values = scan_constraint.get_data_value();
        let delta_values = immediate_values.get_value_bytes().clone();
        let len = delta_values.len();

        if delta_values.iter().any(|value| *value == 0) {
            return None;
        }

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);
            let previous_values = std::slice::from_raw_parts(previous_values_ptr, len);

            current_values
                .iter()
                .zip(previous_values.iter())
                .zip(delta_values.iter())
                .all(|((current_value, previous_value), delta_value)| current_value.rem(*delta_value) == *previous_value)
        }))
    }

    pub fn get_compare_shift_left_by(scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnDelta> {
        let immediate_values = scan_constraint.get_data_value();
        let delta_values = immediate_values.get_value_bytes().clone();
        let len = delta_values.len();

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);
            let previous_values = std::slice::from_raw_parts(previous_values_ptr, len);

            current_values
                .iter()
                .zip(previous_values.iter())
                .zip(delta_values.iter())
                .all(|((current_value, previous_value), delta_value)| current_value.shl(*delta_value) == *previous_value)
        }))
    }

    pub fn get_compare_shift_right_by(scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnDelta> {
        let immediate_values = scan_constraint.get_data_value();
        let delta_values = immediate_values.get_value_bytes().clone();
        let len = delta_values.len();

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);
            let previous_values = std::slice::from_raw_parts(previous_values_ptr, len);

            current_values
                .iter()
                .zip(previous_values.iter())
                .zip(delta_values.iter())
                .all(|((current_value, previous_value), delta_value)| current_value.shr(*delta_value) == *previous_value)
        }))
    }

    pub fn get_compare_logical_and_by(scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnDelta> {
        let immediate_values = scan_constraint.get_data_value();
        let delta_values = immediate_values.get_value_bytes().clone();
        let len = delta_values.len();

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);
            let previous_values = std::slice::from_raw_parts(previous_values_ptr, len);

            current_values
                .iter()
                .zip(previous_values.iter())
                .zip(delta_values.iter())
                .all(|((current_value, previous_value), delta_value)| current_value.bitand(*delta_value) == *previous_value)
        }))
    }

    pub fn get_compare_logical_or_by(scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnDelta> {
        let immediate_values = scan_constraint.get_data_value();
        let delta_values = immediate_values.get_value_bytes().clone();
        let len = delta_values.len();

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);
            let previous_values = std::slice::from_raw_parts(previous_values_ptr, len);

            current_values
                .iter()
                .zip(previous_values.iter())
                .zip(delta_values.iter())
                .all(|((current_value, previous_value), delta_value)| current_value.bitor(*delta_value) == *previous_value)
        }))
    }

    pub fn get_compare_logical_xor_by(scan_constraint: &ScanConstraint) -> Option<ScalarCompareFnDelta> {
        let immediate_values = scan_constraint.get_data_value();
        let delta_values = immediate_values.get_value_bytes().clone();
        let len = delta_values.len();

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);
            let previous_values = std::slice::from_raw_parts(previous_values_ptr, len);

            current_values
                .iter()
                .zip(previous_values.iter())
                .zip(delta_values.iter())
                .all(|((current_value, previous_value), delta_value)| current_value.bitxor(*delta_value) == *previous_value)
        }))
    }
}
