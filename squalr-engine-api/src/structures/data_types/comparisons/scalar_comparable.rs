use crate::structures::scanning::{
    comparisons::scan_function_scalar::{ScalarCompareFnDelta, ScalarCompareFnImmediate, ScalarCompareFnRelative},
    plans::element_scan::snapshot_filter_element_scan_plan::SnapshotFilterElementScanPlan,
};

pub trait ScalarComparable {
    fn get_compare_equal(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate>;

    fn get_compare_not_equal(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate>;

    fn get_compare_greater_than(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate>;

    fn get_compare_greater_than_or_equal(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate>;

    fn get_compare_less_than(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate>;

    fn get_compare_less_than_or_equal(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate>;

    fn get_compare_changed(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnRelative>;

    fn get_compare_unchanged(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnRelative>;

    fn get_compare_increased(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnRelative>;

    fn get_compare_decreased(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnRelative>;

    fn get_compare_increased_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta>;

    fn get_compare_decreased_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta>;

    fn get_compare_multiplied_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta>;

    fn get_compare_divided_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta>;

    fn get_compare_modulo_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta>;

    fn get_compare_shift_left_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta>;

    fn get_compare_shift_right_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta>;

    fn get_compare_logical_and_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta>;

    fn get_compare_logical_or_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta>;

    fn get_compare_logical_xor_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta>;
}
