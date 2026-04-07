use crate::data_types::i24be::data_type_i24be::DataTypeI24be;
use crate::data_types::primitive_data_type_24_bit::PrimitiveDataType24Bit;
use squalr_engine_api::structures::data_types::comparisons::scalar_comparable::ScalarComparable;
use squalr_engine_api::structures::memory::endian::Endian;
use squalr_engine_api::structures::scanning::comparisons::scan_function_scalar::{ScalarCompareFnDelta, ScalarCompareFnImmediate, ScalarCompareFnRelative};
use squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint;

impl ScalarComparable for DataTypeI24be {
    fn get_compare_equal(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnImmediate> {
        PrimitiveDataType24Bit::get_compare_equal_signed(scan_constraint, Endian::Big)
    }

    fn get_compare_not_equal(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnImmediate> {
        PrimitiveDataType24Bit::get_compare_not_equal_signed(scan_constraint, Endian::Big)
    }

    fn get_compare_greater_than(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnImmediate> {
        PrimitiveDataType24Bit::get_compare_greater_than_signed(scan_constraint, Endian::Big)
    }

    fn get_compare_greater_than_or_equal(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnImmediate> {
        PrimitiveDataType24Bit::get_compare_greater_than_or_equal_signed(scan_constraint, Endian::Big)
    }

    fn get_compare_less_than(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnImmediate> {
        PrimitiveDataType24Bit::get_compare_less_than_signed(scan_constraint, Endian::Big)
    }

    fn get_compare_less_than_or_equal(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnImmediate> {
        PrimitiveDataType24Bit::get_compare_less_than_or_equal_signed(scan_constraint, Endian::Big)
    }

    fn get_compare_changed(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnRelative> {
        PrimitiveDataType24Bit::get_compare_changed_signed(scan_constraint, Endian::Big)
    }

    fn get_compare_unchanged(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnRelative> {
        PrimitiveDataType24Bit::get_compare_unchanged_signed(scan_constraint, Endian::Big)
    }

    fn get_compare_increased(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnRelative> {
        PrimitiveDataType24Bit::get_compare_increased_signed(scan_constraint, Endian::Big)
    }

    fn get_compare_decreased(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnRelative> {
        PrimitiveDataType24Bit::get_compare_decreased_signed(scan_constraint, Endian::Big)
    }

    fn get_compare_increased_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        PrimitiveDataType24Bit::get_compare_increased_by_signed(scan_constraint, Endian::Big)
    }

    fn get_compare_decreased_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        PrimitiveDataType24Bit::get_compare_decreased_by_signed(scan_constraint, Endian::Big)
    }

    fn get_compare_multiplied_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        PrimitiveDataType24Bit::get_compare_multiplied_by_signed(scan_constraint, Endian::Big)
    }

    fn get_compare_divided_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        PrimitiveDataType24Bit::get_compare_divided_by_signed(scan_constraint, Endian::Big)
    }

    fn get_compare_modulo_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        PrimitiveDataType24Bit::get_compare_modulo_by_signed(scan_constraint, Endian::Big)
    }

    fn get_compare_shift_left_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        PrimitiveDataType24Bit::get_compare_shift_left_by_signed(scan_constraint, Endian::Big)
    }

    fn get_compare_shift_right_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        PrimitiveDataType24Bit::get_compare_shift_right_by_signed(scan_constraint, Endian::Big)
    }

    fn get_compare_logical_and_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        PrimitiveDataType24Bit::get_compare_logical_and_by_signed(scan_constraint, Endian::Big)
    }

    fn get_compare_logical_or_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        PrimitiveDataType24Bit::get_compare_logical_or_by_signed(scan_constraint, Endian::Big)
    }

    fn get_compare_logical_xor_by(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnDelta> {
        PrimitiveDataType24Bit::get_compare_logical_xor_by_signed(scan_constraint, Endian::Big)
    }
}
