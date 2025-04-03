use crate::structures::data_types::built_in_types::byte_array::data_type_byte_array::DataTypeByteArray;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarComparable;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnDelta;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnImmediate;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnRelative;
use crate::structures::scanning::parameters::scan_parameters::ScanParameters;
use std::cmp::Ordering;

/// Scalar comparison functions for comparing byte arrays. Note that these functions operate on single array values.
/// For performance-critical scans, specialized algorithms are implemented elsewhere.
/// Additionally, many comparison functions for arrays are not defined, such as inequalities or delta scans,
/// and are subject to change. The only clearly defined behaviors are: equal, not equal, changed, and unchanged.
impl ScalarComparable for DataTypeByteArray {
    fn get_compare_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        if let Some(immediate_values) = scan_parameters.get_data_value() {
            let immediate_values = immediate_values.get_value_bytes().clone();
            let len = immediate_values.len();

            Some(Box::new(move |current_values_ptr| unsafe {
                let current_values = std::slice::from_raw_parts(current_values_ptr, len);
                current_values == immediate_values
            }))
        } else {
            None
        }
    }

    fn get_compare_not_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        if let Some(immediate_values) = scan_parameters.get_data_value() {
            let immediate_values = immediate_values.get_value_bytes().clone();
            let len = immediate_values.len();

            Some(Box::new(move |current_values_ptr| unsafe {
                let current_values = std::slice::from_raw_parts(current_values_ptr, len);
                current_values != immediate_values
            }))
        } else {
            None
        }
    }

    fn get_compare_greater_than(
        &self,
        scan_parameters: &ScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        if let Some(immediate_values) = scan_parameters.get_data_value() {
            let immediate_values = immediate_values.get_value_bytes().clone();
            let len = immediate_values.len();

            Some(Box::new(move |current_values_ptr| unsafe {
                let current_values = std::slice::from_raw_parts(current_values_ptr, len);
                current_values
                    .iter()
                    .zip(immediate_values.iter())
                    .all(|(current_values, immediate_values)| current_values > immediate_values)
            }))
        } else {
            None
        }
    }

    fn get_compare_greater_than_or_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        if let Some(immediate_values) = scan_parameters.get_data_value() {
            let immediate_values = immediate_values.get_value_bytes().clone();
            let len = immediate_values.len();

            Some(Box::new(move |current_values_ptr| unsafe {
                let current_values = std::slice::from_raw_parts(current_values_ptr, len);

                current_values.cmp(&immediate_values) == Ordering::Greater || current_values == immediate_values
            }))
        } else {
            None
        }
    }

    fn get_compare_less_than(
        &self,
        scan_parameters: &ScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        if let Some(immediate_values) = scan_parameters.get_data_value() {
            let immediate_values = immediate_values.get_value_bytes().clone();
            let len = immediate_values.len();

            Some(Box::new(move |current_values_ptr| unsafe {
                let current_values = std::slice::from_raw_parts(current_values_ptr, len);

                current_values.cmp(&immediate_values) == Ordering::Less
            }))
        } else {
            None
        }
    }

    fn get_compare_less_than_or_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        if let Some(immediate_values) = scan_parameters.get_data_value() {
            let immediate_values = immediate_values.get_value_bytes().clone();
            let len = immediate_values.len();

            Some(Box::new(move |current_values_ptr| unsafe {
                let current_values = std::slice::from_raw_parts(current_values_ptr, len);

                current_values.cmp(&immediate_values) == Ordering::Less || current_values == immediate_values
            }))
        } else {
            None
        }
    }

    fn get_compare_changed(
        &self,
        scan_parameters: &ScanParameters,
    ) -> Option<ScalarCompareFnRelative> {
        let len = scan_parameters.get_optimized_data_type().get_size_in_bytes() as usize;

        Some(Box::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);
            let previous_values = std::slice::from_raw_parts(previous_values_ptr, len);
            current_values != previous_values
        }))
    }

    fn get_compare_unchanged(
        &self,
        scan_parameters: &ScanParameters,
    ) -> Option<ScalarCompareFnRelative> {
        let len = scan_parameters.get_optimized_data_type().get_size_in_bytes() as usize;

        Some(Box::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);
            let previous_values = std::slice::from_raw_parts(previous_values_ptr, len);

            current_values == previous_values
        }))
    }

    fn get_compare_increased(
        &self,
        scan_parameters: &ScanParameters,
    ) -> Option<ScalarCompareFnRelative> {
        let len = scan_parameters.get_optimized_data_type().get_size_in_bytes() as usize;

        Some(Box::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);
            let previous_values = std::slice::from_raw_parts(previous_values_ptr, len);

            current_values
                .iter()
                .zip(previous_values.iter())
                .all(|(current_value, previous_value)| current_value > previous_value)
        }))
    }

    fn get_compare_decreased(
        &self,
        scan_parameters: &ScanParameters,
    ) -> Option<ScalarCompareFnRelative> {
        let len = scan_parameters.get_optimized_data_type().get_size_in_bytes() as usize;

        Some(Box::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = std::slice::from_raw_parts(current_values_ptr, len);
            let previous_values = std::slice::from_raw_parts(previous_values_ptr, len);

            current_values
                .iter()
                .zip(previous_values.iter())
                .all(|(current_value, previous_value)| current_value < previous_value)
        }))
    }

    fn get_compare_increased_by(
        &self,
        scan_parameters: &ScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        if let Some(delta_values) = scan_parameters.get_data_value() {
            let delta_values = delta_values.get_value_bytes().clone();
            let len = delta_values.len();

            Some(Box::new(move |current_values_ptr, previous_values_ptr| unsafe {
                let current_values = std::slice::from_raw_parts(current_values_ptr, len);
                let previous_values = std::slice::from_raw_parts(previous_values_ptr, len);

                current_values
                    .iter()
                    .zip(previous_values.iter())
                    .zip(delta_values.iter())
                    .all(|((current_value, previous_value), delta_value)| current_value.wrapping_add(*delta_value) == *previous_value)
            }))
        } else {
            None
        }
    }

    fn get_compare_decreased_by(
        &self,
        scan_parameters: &ScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        if let Some(delta_values) = scan_parameters.get_data_value() {
            let delta_values = delta_values.get_value_bytes().clone();
            let len = delta_values.len();

            Some(Box::new(move |current_values_ptr, previous_values_ptr| unsafe {
                let current_values = std::slice::from_raw_parts(current_values_ptr, len);
                let previous_values = std::slice::from_raw_parts(previous_values_ptr, len);

                current_values
                    .iter()
                    .zip(previous_values.iter())
                    .zip(delta_values.iter())
                    .all(|((current_value, previous_value), delta_value)| current_value.wrapping_sub(*delta_value) == *previous_value)
            }))
        } else {
            None
        }
    }
}
