use crate::structures::data_types::built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarComparable;
use crate::structures::data_types::comparisons::scalar_comparisons_byte_array::ScalarComparisonsByteArray;
use crate::structures::scanning::comparisons::scan_function_scalar::{ScalarCompareFnDelta, ScalarCompareFnImmediate, ScalarCompareFnRelative};
use crate::structures::scanning::plans::element_scan::snapshot_filter_element_scan_plan::SnapshotFilterElementScanPlan;

impl ScalarComparable for DataTypeStringUtf8 {
    fn get_compare_equal(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsByteArray::get_compare_equal(snapshot_filter_element_scan_plan)
    }

    fn get_compare_not_equal(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsByteArray::get_compare_not_equal(snapshot_filter_element_scan_plan)
    }

    fn get_compare_greater_than(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsByteArray::get_compare_greater_than(snapshot_filter_element_scan_plan)
    }

    fn get_compare_greater_than_or_equal(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsByteArray::get_compare_greater_than_or_equal(snapshot_filter_element_scan_plan)
    }

    fn get_compare_less_than(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsByteArray::get_compare_less_than(snapshot_filter_element_scan_plan)
    }

    fn get_compare_less_than_or_equal(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsByteArray::get_compare_less_than_or_equal(snapshot_filter_element_scan_plan)
    }

    fn get_compare_changed(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsByteArray::get_compare_changed(snapshot_filter_element_scan_plan)
    }

    fn get_compare_unchanged(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsByteArray::get_compare_unchanged(snapshot_filter_element_scan_plan)
    }

    fn get_compare_increased(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsByteArray::get_compare_increased(snapshot_filter_element_scan_plan)
    }

    fn get_compare_decreased(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsByteArray::get_compare_decreased(snapshot_filter_element_scan_plan)
    }

    fn get_compare_increased_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_increased_by(snapshot_filter_element_scan_plan)
    }

    fn get_compare_decreased_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_decreased_by(snapshot_filter_element_scan_plan)
    }

    fn get_compare_multiplied_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_multiplied_by(snapshot_filter_element_scan_plan)
    }

    fn get_compare_divided_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_divided_by(snapshot_filter_element_scan_plan)
    }

    fn get_compare_modulo_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_modulo_by(snapshot_filter_element_scan_plan)
    }

    fn get_compare_shift_left_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_shift_left_by(snapshot_filter_element_scan_plan)
    }

    fn get_compare_shift_right_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_shift_right_by(snapshot_filter_element_scan_plan)
    }

    fn get_compare_logical_and_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_logical_and_by(snapshot_filter_element_scan_plan)
    }

    fn get_compare_logical_or_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_logical_or_by(snapshot_filter_element_scan_plan)
    }

    fn get_compare_logical_xor_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsByteArray::get_compare_logical_xor_by(snapshot_filter_element_scan_plan)
    }
}
