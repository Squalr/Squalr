use crate::structures::data_types::built_in_types::bool32::data_type_bool32::DataTypeBool32;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarComparable;
use crate::structures::data_types::comparisons::scalar_comparisons_integer::ScalarComparisonsInteger;
use crate::structures::scanning::comparisons::scan_function_scalar::{ScalarCompareFnDelta, ScalarCompareFnImmediate, ScalarCompareFnRelative};
use crate::structures::scanning::constraints::scan_constraint::ScanConstraint;

// For a bool32 the underlying primitive is a u32.
type PrimitiveType = u32;

impl ScalarComparable for DataTypeBool32 {
    fn get_compare_equal(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsInteger::get_compare_equal::<PrimitiveType>(scan_constraint)
    }

    fn get_compare_not_equal(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsInteger::get_compare_not_equal::<PrimitiveType>(scan_constraint)
    }

    fn get_compare_greater_than(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsInteger::get_compare_greater_than::<PrimitiveType>(scan_constraint)
    }

    fn get_compare_greater_than_or_equal(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsInteger::get_compare_greater_than_or_equal::<PrimitiveType>(scan_constraint)
    }

    fn get_compare_less_than(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsInteger::get_compare_less_than::<PrimitiveType>(scan_constraint)
    }

    fn get_compare_less_than_or_equal(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsInteger::get_compare_less_than_or_equal::<PrimitiveType>(scan_constraint)
    }

    fn get_compare_changed(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsInteger::get_compare_changed::<PrimitiveType>(scan_constraint)
    }

    fn get_compare_unchanged(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsInteger::get_compare_unchanged::<PrimitiveType>(scan_constraint)
    }

    fn get_compare_increased(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsInteger::get_compare_increased::<PrimitiveType>(scan_constraint)
    }

    fn get_compare_decreased(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsInteger::get_compare_decreased::<PrimitiveType>(scan_constraint)
    }

    fn get_compare_increased_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsInteger::get_compare_increased_by::<PrimitiveType>(scan_constraint)
    }

    fn get_compare_decreased_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsInteger::get_compare_decreased_by::<PrimitiveType>(scan_constraint)
    }

    fn get_compare_multiplied_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsInteger::get_compare_multiplied_by::<PrimitiveType>(scan_constraint)
    }

    fn get_compare_divided_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsInteger::get_compare_divided_by::<PrimitiveType>(scan_constraint)
    }

    fn get_compare_modulo_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsInteger::get_compare_modulo_by::<PrimitiveType>(scan_constraint)
    }

    fn get_compare_shift_left_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsInteger::get_compare_shift_left_by::<PrimitiveType>(scan_constraint)
    }

    fn get_compare_shift_right_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsInteger::get_compare_shift_right_by::<PrimitiveType>(scan_constraint)
    }

    fn get_compare_logical_and_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsInteger::get_compare_logical_and_by::<PrimitiveType>(scan_constraint)
    }

    fn get_compare_logical_or_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsInteger::get_compare_logical_or_by::<PrimitiveType>(scan_constraint)
    }

    fn get_compare_logical_xor_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsInteger::get_compare_logical_xor_by::<PrimitiveType>(scan_constraint)
    }
}
