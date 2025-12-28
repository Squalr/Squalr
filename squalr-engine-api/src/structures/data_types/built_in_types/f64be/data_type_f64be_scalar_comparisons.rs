use crate::structures::data_types::built_in_types::f64be::data_type_f64be::DataTypeF64be;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarComparable;
use crate::structures::data_types::comparisons::scalar_comparisons_float_big_endian::ScalarComparisonsFloatBigEndian;
use crate::structures::scanning::comparisons::scan_function_scalar::{ScalarCompareFnDelta, ScalarCompareFnImmediate, ScalarCompareFnRelative};
use crate::structures::scanning::plans::element_scan::snapshot_filter_element_scan_plan::SnapshotFilterElementScanPlan;

type PrimitiveType = f64;

impl ScalarComparable for DataTypeF64be {
    fn get_compare_equal(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsFloatBigEndian::get_compare_equal::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_not_equal(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsFloatBigEndian::get_compare_not_equal::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_greater_than(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsFloatBigEndian::get_compare_greater_than::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_greater_than_or_equal(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsFloatBigEndian::get_compare_greater_than_or_equal::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_less_than(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsFloatBigEndian::get_compare_less_than::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_less_than_or_equal(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsFloatBigEndian::get_compare_less_than_or_equal::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_changed(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsFloatBigEndian::get_compare_changed::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_unchanged(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsFloatBigEndian::get_compare_unchanged::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_increased(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsFloatBigEndian::get_compare_increased::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_decreased(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsFloatBigEndian::get_compare_decreased::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_increased_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsFloatBigEndian::get_compare_increased_by::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_decreased_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsFloatBigEndian::get_compare_decreased_by::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_multiplied_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsFloatBigEndian::get_compare_multiplied_by::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_divided_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsFloatBigEndian::get_compare_divided_by::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_modulo_by(
        &self,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsFloatBigEndian::get_compare_modulo_by::<PrimitiveType>(snapshot_filter_element_scan_plan)
    }

    fn get_compare_shift_left_by(
        &self,
        _snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        None
    }

    fn get_compare_shift_right_by(
        &self,
        _snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        None
    }

    fn get_compare_logical_and_by(
        &self,
        _snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        None
    }

    fn get_compare_logical_or_by(
        &self,
        _snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        None
    }

    fn get_compare_logical_xor_by(
        &self,
        _snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnDelta> {
        None
    }
}
