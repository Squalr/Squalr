use crate::structures::data_types::built_in_types::data_type_ref::data_type_data_type_ref::DataTypeRefDataType;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarComparable;
use crate::structures::scanning::comparisons::scan_function_scalar::{ScalarCompareFnDelta, ScalarCompareFnImmediate, ScalarCompareFnRelative};
use crate::structures::scanning::parameters::mapped::mapped_scan_parameters::MappedScanParameters;

/// Comparisons for the 'data type ref' data type are not supported.
impl ScalarComparable for DataTypeRefDataType {
    fn get_compare_equal(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        None
    }

    fn get_compare_not_equal(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        None
    }

    fn get_compare_greater_than(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        None
    }

    fn get_compare_greater_than_or_equal(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        None
    }

    fn get_compare_less_than(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        None
    }

    fn get_compare_less_than_or_equal(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        None
    }

    fn get_compare_changed(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnRelative> {
        None
    }

    fn get_compare_unchanged(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnRelative> {
        None
    }

    fn get_compare_increased(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnRelative> {
        None
    }

    fn get_compare_decreased(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnRelative> {
        None
    }

    fn get_compare_increased_by(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        None
    }

    fn get_compare_decreased_by(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        None
    }

    fn get_compare_multiplied_by(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        None
    }

    fn get_compare_divided_by(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        None
    }

    fn get_compare_modulo_by(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        None
    }

    fn get_compare_shift_left_by(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        None
    }

    fn get_compare_shift_right_by(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        None
    }

    fn get_compare_logical_and_by(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        None
    }

    fn get_compare_logical_or_by(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        None
    }

    fn get_compare_logical_xor_by(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        None
    }
}
