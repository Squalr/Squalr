use crate::structures::data_types::built_in_types::i8::data_type_i8::DataTypeI8;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarComparable;
use crate::structures::data_types::comparisons::scalar_comparisons_integer::ScalarComparisonsInteger;
use crate::structures::scanning::comparisons::scan_function_scalar::{ScalarCompareFnDelta, ScalarCompareFnImmediate, ScalarCompareFnRelative};
use crate::structures::scanning::plans::element_scan::snapshot_filter_element_scan_plan::SnapshotFilterElementScanPlan;

type PrimitiveType = i8;

impl ScalarComparable for DataTypeI8 {
    fn get_compare_equal(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsInteger::get_compare_equal::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_not_equal(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsInteger::get_compare_not_equal::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_greater_than(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsInteger::get_compare_greater_than::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_greater_than_or_equal(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsInteger::get_compare_greater_than_or_equal::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_less_than(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsInteger::get_compare_less_than::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_less_than_or_equal(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsInteger::get_compare_less_than_or_equal::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_changed(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsInteger::get_compare_changed::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_unchanged(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsInteger::get_compare_unchanged::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_increased(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsInteger::get_compare_increased::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_decreased(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsInteger::get_compare_decreased::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_increased_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsInteger::get_compare_increased_by::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_decreased_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsInteger::get_compare_decreased_by::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_multiplied_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsInteger::get_compare_multiplied_by::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_divided_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsInteger::get_compare_divided_by::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_modulo_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsInteger::get_compare_modulo_by::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_shift_left_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsInteger::get_compare_shift_left_by::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_shift_right_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsInteger::get_compare_shift_right_by::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_logical_and_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsInteger::get_compare_logical_and_by::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_logical_or_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsInteger::get_compare_logical_or_by::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_logical_xor_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsInteger::get_compare_logical_xor_by::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }
}
