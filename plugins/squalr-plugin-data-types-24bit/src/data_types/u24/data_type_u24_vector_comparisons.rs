use crate::data_types::data_type_24_bit_vector_comparisons;
use crate::data_types::u24::data_type_u24::DataTypeU24;
use squalr_engine_api::structures::data_types::comparisons::vector_comparable::VectorComparable;
use squalr_engine_api::structures::memory::endian::Endian;
use squalr_engine_api::structures::scanning::comparisons::scan_function_vector::{
    VectorCompareFnDelta16, VectorCompareFnDelta32, VectorCompareFnDelta64, VectorCompareFnImmediate16, VectorCompareFnImmediate32, VectorCompareFnImmediate64,
    VectorCompareFnRelative16, VectorCompareFnRelative32, VectorCompareFnRelative64,
};
use squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint;

const BYTE_COUNT_64: usize = 64;
const BYTE_COUNT_32: usize = 32;
const BYTE_COUNT_16: usize = 16;

impl VectorComparable for DataTypeU24 {
    fn get_vector_compare_equal_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate64> {
        data_type_24_bit_vector_comparisons::get_vector_compare_equal_unsigned::<{ BYTE_COUNT_64 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_equal_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate32> {
        data_type_24_bit_vector_comparisons::get_vector_compare_equal_unsigned::<{ BYTE_COUNT_32 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_equal_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate16> {
        data_type_24_bit_vector_comparisons::get_vector_compare_equal_unsigned::<{ BYTE_COUNT_16 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_not_equal_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate64> {
        data_type_24_bit_vector_comparisons::get_vector_compare_not_equal_unsigned::<{ BYTE_COUNT_64 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_not_equal_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate32> {
        data_type_24_bit_vector_comparisons::get_vector_compare_not_equal_unsigned::<{ BYTE_COUNT_32 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_not_equal_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate16> {
        data_type_24_bit_vector_comparisons::get_vector_compare_not_equal_unsigned::<{ BYTE_COUNT_16 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_greater_than_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate64> {
        data_type_24_bit_vector_comparisons::get_vector_compare_greater_than_unsigned::<{ BYTE_COUNT_64 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_greater_than_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate32> {
        data_type_24_bit_vector_comparisons::get_vector_compare_greater_than_unsigned::<{ BYTE_COUNT_32 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_greater_than_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate16> {
        data_type_24_bit_vector_comparisons::get_vector_compare_greater_than_unsigned::<{ BYTE_COUNT_16 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_greater_than_or_equal_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate64> {
        data_type_24_bit_vector_comparisons::get_vector_compare_greater_than_or_equal_unsigned::<{ BYTE_COUNT_64 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_greater_than_or_equal_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate32> {
        data_type_24_bit_vector_comparisons::get_vector_compare_greater_than_or_equal_unsigned::<{ BYTE_COUNT_32 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_greater_than_or_equal_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate16> {
        data_type_24_bit_vector_comparisons::get_vector_compare_greater_than_or_equal_unsigned::<{ BYTE_COUNT_16 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_less_than_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate64> {
        data_type_24_bit_vector_comparisons::get_vector_compare_less_than_unsigned::<{ BYTE_COUNT_64 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_less_than_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate32> {
        data_type_24_bit_vector_comparisons::get_vector_compare_less_than_unsigned::<{ BYTE_COUNT_32 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_less_than_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate16> {
        data_type_24_bit_vector_comparisons::get_vector_compare_less_than_unsigned::<{ BYTE_COUNT_16 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_less_than_or_equal_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate64> {
        data_type_24_bit_vector_comparisons::get_vector_compare_less_than_or_equal_unsigned::<{ BYTE_COUNT_64 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_less_than_or_equal_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate32> {
        data_type_24_bit_vector_comparisons::get_vector_compare_less_than_or_equal_unsigned::<{ BYTE_COUNT_32 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_less_than_or_equal_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate16> {
        data_type_24_bit_vector_comparisons::get_vector_compare_less_than_or_equal_unsigned::<{ BYTE_COUNT_16 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_changed_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative64> {
        data_type_24_bit_vector_comparisons::get_vector_compare_changed_unsigned::<{ BYTE_COUNT_64 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_changed_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative32> {
        data_type_24_bit_vector_comparisons::get_vector_compare_changed_unsigned::<{ BYTE_COUNT_32 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_changed_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative16> {
        data_type_24_bit_vector_comparisons::get_vector_compare_changed_unsigned::<{ BYTE_COUNT_16 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_unchanged_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative64> {
        data_type_24_bit_vector_comparisons::get_vector_compare_unchanged_unsigned::<{ BYTE_COUNT_64 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_unchanged_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative32> {
        data_type_24_bit_vector_comparisons::get_vector_compare_unchanged_unsigned::<{ BYTE_COUNT_32 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_unchanged_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative16> {
        data_type_24_bit_vector_comparisons::get_vector_compare_unchanged_unsigned::<{ BYTE_COUNT_16 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_increased_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative64> {
        data_type_24_bit_vector_comparisons::get_vector_compare_increased_unsigned::<{ BYTE_COUNT_64 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_increased_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative32> {
        data_type_24_bit_vector_comparisons::get_vector_compare_increased_unsigned::<{ BYTE_COUNT_32 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_increased_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative16> {
        data_type_24_bit_vector_comparisons::get_vector_compare_increased_unsigned::<{ BYTE_COUNT_16 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_decreased_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative64> {
        data_type_24_bit_vector_comparisons::get_vector_compare_decreased_unsigned::<{ BYTE_COUNT_64 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_decreased_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative32> {
        data_type_24_bit_vector_comparisons::get_vector_compare_decreased_unsigned::<{ BYTE_COUNT_32 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_decreased_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative16> {
        data_type_24_bit_vector_comparisons::get_vector_compare_decreased_unsigned::<{ BYTE_COUNT_16 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_increased_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64> {
        data_type_24_bit_vector_comparisons::get_vector_compare_increased_by_unsigned::<{ BYTE_COUNT_64 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_increased_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32> {
        data_type_24_bit_vector_comparisons::get_vector_compare_increased_by_unsigned::<{ BYTE_COUNT_32 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_increased_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16> {
        data_type_24_bit_vector_comparisons::get_vector_compare_increased_by_unsigned::<{ BYTE_COUNT_16 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_decreased_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64> {
        data_type_24_bit_vector_comparisons::get_vector_compare_decreased_by_unsigned::<{ BYTE_COUNT_64 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_decreased_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32> {
        data_type_24_bit_vector_comparisons::get_vector_compare_decreased_by_unsigned::<{ BYTE_COUNT_32 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_decreased_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16> {
        data_type_24_bit_vector_comparisons::get_vector_compare_decreased_by_unsigned::<{ BYTE_COUNT_16 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_multiplied_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64> {
        data_type_24_bit_vector_comparisons::get_vector_compare_multiplied_by_unsigned::<{ BYTE_COUNT_64 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_multiplied_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32> {
        data_type_24_bit_vector_comparisons::get_vector_compare_multiplied_by_unsigned::<{ BYTE_COUNT_32 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_multiplied_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16> {
        data_type_24_bit_vector_comparisons::get_vector_compare_multiplied_by_unsigned::<{ BYTE_COUNT_16 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_divided_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64> {
        data_type_24_bit_vector_comparisons::get_vector_compare_divided_by_unsigned::<{ BYTE_COUNT_64 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_divided_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32> {
        data_type_24_bit_vector_comparisons::get_vector_compare_divided_by_unsigned::<{ BYTE_COUNT_32 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_divided_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16> {
        data_type_24_bit_vector_comparisons::get_vector_compare_divided_by_unsigned::<{ BYTE_COUNT_16 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_modulo_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64> {
        data_type_24_bit_vector_comparisons::get_vector_compare_modulo_by_unsigned::<{ BYTE_COUNT_64 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_modulo_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32> {
        data_type_24_bit_vector_comparisons::get_vector_compare_modulo_by_unsigned::<{ BYTE_COUNT_32 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_modulo_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16> {
        data_type_24_bit_vector_comparisons::get_vector_compare_modulo_by_unsigned::<{ BYTE_COUNT_16 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_shift_left_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64> {
        data_type_24_bit_vector_comparisons::get_vector_compare_shift_left_by_unsigned::<{ BYTE_COUNT_64 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_shift_left_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32> {
        data_type_24_bit_vector_comparisons::get_vector_compare_shift_left_by_unsigned::<{ BYTE_COUNT_32 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_shift_left_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16> {
        data_type_24_bit_vector_comparisons::get_vector_compare_shift_left_by_unsigned::<{ BYTE_COUNT_16 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_shift_right_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64> {
        data_type_24_bit_vector_comparisons::get_vector_compare_shift_right_by_unsigned::<{ BYTE_COUNT_64 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_shift_right_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32> {
        data_type_24_bit_vector_comparisons::get_vector_compare_shift_right_by_unsigned::<{ BYTE_COUNT_32 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_shift_right_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16> {
        data_type_24_bit_vector_comparisons::get_vector_compare_shift_right_by_unsigned::<{ BYTE_COUNT_16 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_logical_and_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64> {
        data_type_24_bit_vector_comparisons::get_vector_compare_logical_and_by_unsigned::<{ BYTE_COUNT_64 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_logical_and_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32> {
        data_type_24_bit_vector_comparisons::get_vector_compare_logical_and_by_unsigned::<{ BYTE_COUNT_32 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_logical_and_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16> {
        data_type_24_bit_vector_comparisons::get_vector_compare_logical_and_by_unsigned::<{ BYTE_COUNT_16 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_logical_or_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64> {
        data_type_24_bit_vector_comparisons::get_vector_compare_logical_or_by_unsigned::<{ BYTE_COUNT_64 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_logical_or_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32> {
        data_type_24_bit_vector_comparisons::get_vector_compare_logical_or_by_unsigned::<{ BYTE_COUNT_32 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_logical_or_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16> {
        data_type_24_bit_vector_comparisons::get_vector_compare_logical_or_by_unsigned::<{ BYTE_COUNT_16 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_logical_xor_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64> {
        data_type_24_bit_vector_comparisons::get_vector_compare_logical_xor_by_unsigned::<{ BYTE_COUNT_64 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_logical_xor_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32> {
        data_type_24_bit_vector_comparisons::get_vector_compare_logical_xor_by_unsigned::<{ BYTE_COUNT_32 }>(scan_constraint, Endian::Little)
    }

    fn get_vector_compare_logical_xor_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16> {
        data_type_24_bit_vector_comparisons::get_vector_compare_logical_xor_by_unsigned::<{ BYTE_COUNT_16 }>(scan_constraint, Endian::Little)
    }
}
