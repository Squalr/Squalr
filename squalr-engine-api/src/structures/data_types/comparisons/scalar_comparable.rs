use crate::structures::scanning::{
    comparisons::scan_function_scalar::{ScalarCompareFnDelta, ScalarCompareFnImmediate, ScalarCompareFnRelative},
    parameters::mapped::mapped_scan_parameters::MappedScanParameters,
};

pub trait ScalarComparable {
    fn get_compare_equal(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate>;
    fn get_compare_not_equal(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate>;
    fn get_compare_greater_than(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate>;
    fn get_compare_greater_than_or_equal(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate>;
    fn get_compare_less_than(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate>;
    fn get_compare_less_than_or_equal(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate>;

    fn get_compare_changed(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnRelative>;
    fn get_compare_unchanged(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnRelative>;
    fn get_compare_increased(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnRelative>;
    fn get_compare_decreased(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnRelative>;

    fn get_compare_increased_by(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta>;
    fn get_compare_decreased_by(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta>;
    fn get_compare_multiplied_by(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta>;
    fn get_compare_divided_by(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta>;
    fn get_compare_modulo_by(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta>;
    fn get_compare_shift_left_by(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta>;
    fn get_compare_shift_right_by(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta>;
    fn get_compare_logical_and_by(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta>;
    fn get_compare_logical_or_by(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta>;
    fn get_compare_logical_xor_by(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta>;
}
