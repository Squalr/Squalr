use crate::structures::scanning::{
    comparisons::scan_function_scalar::{ScalarCompareFnDelta, ScalarCompareFnImmediate, ScalarCompareFnRelative},
    constraints::optimized_scan_constraint::OptimizedScanConstraint,
};

pub trait ScalarComparable {
    fn get_compare_equal(
        &self,
        mapped_scan_parameters: &OptimizedScanConstraint,
    ) -> Option<ScalarCompareFnImmediate>;
    fn get_compare_not_equal(
        &self,
        mapped_scan_parameters: &OptimizedScanConstraint,
    ) -> Option<ScalarCompareFnImmediate>;
    fn get_compare_greater_than(
        &self,
        mapped_scan_parameters: &OptimizedScanConstraint,
    ) -> Option<ScalarCompareFnImmediate>;
    fn get_compare_greater_than_or_equal(
        &self,
        mapped_scan_parameters: &OptimizedScanConstraint,
    ) -> Option<ScalarCompareFnImmediate>;
    fn get_compare_less_than(
        &self,
        mapped_scan_parameters: &OptimizedScanConstraint,
    ) -> Option<ScalarCompareFnImmediate>;
    fn get_compare_less_than_or_equal(
        &self,
        mapped_scan_parameters: &OptimizedScanConstraint,
    ) -> Option<ScalarCompareFnImmediate>;

    fn get_compare_changed(
        &self,
        mapped_scan_parameters: &OptimizedScanConstraint,
    ) -> Option<ScalarCompareFnRelative>;
    fn get_compare_unchanged(
        &self,
        mapped_scan_parameters: &OptimizedScanConstraint,
    ) -> Option<ScalarCompareFnRelative>;
    fn get_compare_increased(
        &self,
        mapped_scan_parameters: &OptimizedScanConstraint,
    ) -> Option<ScalarCompareFnRelative>;
    fn get_compare_decreased(
        &self,
        mapped_scan_parameters: &OptimizedScanConstraint,
    ) -> Option<ScalarCompareFnRelative>;

    fn get_compare_increased_by(
        &self,
        mapped_scan_parameters: &OptimizedScanConstraint,
    ) -> Option<ScalarCompareFnDelta>;
    fn get_compare_decreased_by(
        &self,
        mapped_scan_parameters: &OptimizedScanConstraint,
    ) -> Option<ScalarCompareFnDelta>;
    fn get_compare_multiplied_by(
        &self,
        mapped_scan_parameters: &OptimizedScanConstraint,
    ) -> Option<ScalarCompareFnDelta>;
    fn get_compare_divided_by(
        &self,
        mapped_scan_parameters: &OptimizedScanConstraint,
    ) -> Option<ScalarCompareFnDelta>;
    fn get_compare_modulo_by(
        &self,
        mapped_scan_parameters: &OptimizedScanConstraint,
    ) -> Option<ScalarCompareFnDelta>;
    fn get_compare_shift_left_by(
        &self,
        mapped_scan_parameters: &OptimizedScanConstraint,
    ) -> Option<ScalarCompareFnDelta>;
    fn get_compare_shift_right_by(
        &self,
        mapped_scan_parameters: &OptimizedScanConstraint,
    ) -> Option<ScalarCompareFnDelta>;
    fn get_compare_logical_and_by(
        &self,
        mapped_scan_parameters: &OptimizedScanConstraint,
    ) -> Option<ScalarCompareFnDelta>;
    fn get_compare_logical_or_by(
        &self,
        mapped_scan_parameters: &OptimizedScanConstraint,
    ) -> Option<ScalarCompareFnDelta>;
    fn get_compare_logical_xor_by(
        &self,
        mapped_scan_parameters: &OptimizedScanConstraint,
    ) -> Option<ScalarCompareFnDelta>;
}
