use crate::structures::data_types::built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarComparable;
use crate::structures::data_types::comparisons::scalar_comparisons_byte_array::ScalarComparisonsByteArray;
use crate::structures::scanning::comparisons::scan_function_scalar::{ScalarCompareFnDelta, ScalarCompareFnImmediate, ScalarCompareFnRelative};
use crate::structures::scanning::constraints::scan_constraint::ScanConstraint;

impl ScalarComparable for DataTypeStringUtf8 {
    fn get_compare_equal(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsByteArray::get_compare_equal(scan_constraint)
    }

    fn get_compare_not_equal(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsByteArray::get_compare_not_equal(scan_constraint)
    }

    fn get_compare_greater_than(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsByteArray::get_compare_greater_than(scan_constraint)
    }

    fn get_compare_greater_than_or_equal(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsByteArray::get_compare_greater_than_or_equal(scan_constraint)
    }

    fn get_compare_less_than(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsByteArray::get_compare_less_than(scan_constraint)
    }

    fn get_compare_less_than_or_equal(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsByteArray::get_compare_less_than_or_equal(scan_constraint)
    }

    fn get_compare_changed(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsByteArray::get_compare_changed(scan_constraint)
    }

    fn get_compare_unchanged(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsByteArray::get_compare_unchanged(scan_constraint)
    }

    fn get_compare_increased(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsByteArray::get_compare_increased(scan_constraint)
    }

    fn get_compare_decreased(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsByteArray::get_compare_decreased(scan_constraint)
    }

    fn get_compare_increased_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_increased_by(scan_constraint)
    }

    fn get_compare_decreased_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_decreased_by(scan_constraint)
    }

    fn get_compare_multiplied_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_multiplied_by(scan_constraint)
    }

    fn get_compare_divided_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_divided_by(scan_constraint)
    }

    fn get_compare_modulo_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_modulo_by(scan_constraint)
    }

    fn get_compare_shift_left_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_shift_left_by(scan_constraint)
    }

    fn get_compare_shift_right_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_shift_right_by(scan_constraint)
    }

    fn get_compare_logical_and_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_logical_and_by(scan_constraint)
    }

    fn get_compare_logical_or_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_logical_or_by(scan_constraint)
    }

    fn get_compare_logical_xor_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_logical_xor_by(scan_constraint)
    }
}
