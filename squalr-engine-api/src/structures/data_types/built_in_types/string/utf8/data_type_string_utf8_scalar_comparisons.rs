use crate::structures::data_types::built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarComparable;
use crate::structures::data_types::comparisons::scalar_comparisons_byte_array::ScalarComparisonsByteArray;
use crate::structures::scanning::comparisons::scan_function_scalar::{ScalarCompareFnDelta, ScalarCompareFnImmediate, ScalarCompareFnRelative};
use crate::structures::scanning::parameters::mapped::mapped_scan_parameters::MappedScanParameters;

impl ScalarComparable for DataTypeStringUtf8 {
    fn get_compare_equal(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsByteArray::get_compare_equal(mapped_scan_parameters)
    }

    fn get_compare_not_equal(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsByteArray::get_compare_not_equal(mapped_scan_parameters)
    }

    fn get_compare_greater_than(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsByteArray::get_compare_greater_than(mapped_scan_parameters)
    }

    fn get_compare_greater_than_or_equal(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsByteArray::get_compare_greater_than_or_equal(mapped_scan_parameters)
    }

    fn get_compare_less_than(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsByteArray::get_compare_less_than(mapped_scan_parameters)
    }

    fn get_compare_less_than_or_equal(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsByteArray::get_compare_less_than_or_equal(mapped_scan_parameters)
    }

    fn get_compare_changed(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsByteArray::get_compare_changed(mapped_scan_parameters)
    }

    fn get_compare_unchanged(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsByteArray::get_compare_unchanged(mapped_scan_parameters)
    }

    fn get_compare_increased(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsByteArray::get_compare_increased(mapped_scan_parameters)
    }

    fn get_compare_decreased(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsByteArray::get_compare_decreased(mapped_scan_parameters)
    }

    fn get_compare_increased_by(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_increased_by(mapped_scan_parameters)
    }

    fn get_compare_decreased_by(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_decreased_by(mapped_scan_parameters)
    }

    fn get_compare_multiplied_by(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_multiplied_by(mapped_scan_parameters)
    }

    fn get_compare_divided_by(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_divided_by(mapped_scan_parameters)
    }

    fn get_compare_modulo_by(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_modulo_by(mapped_scan_parameters)
    }

    fn get_compare_shift_left_by(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_shift_left_by(mapped_scan_parameters)
    }

    fn get_compare_shift_right_by(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_shift_right_by(mapped_scan_parameters)
    }

    fn get_compare_logical_and_by(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_logical_and_by(mapped_scan_parameters)
    }

    fn get_compare_logical_or_by(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_logical_or_by(mapped_scan_parameters)
    }

    fn get_compare_logical_xor_by(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_logical_xor_by(mapped_scan_parameters)
    }
}
