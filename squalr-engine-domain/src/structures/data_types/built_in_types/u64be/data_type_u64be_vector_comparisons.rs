use crate::structures::data_types::built_in_types::u64be::data_type_u64be::DataTypeU64be;
use crate::structures::data_types::comparisons::vector_comparable::VectorComparable;
use crate::structures::data_types::comparisons::vector_comparisons_integer_big_endian::VectorComparisonsIntegerBigEndian;
use crate::structures::scanning::comparisons::scan_function_vector::{
    VectorCompareFnDelta16, VectorCompareFnDelta32, VectorCompareFnDelta64, VectorCompareFnImmediate16, VectorCompareFnImmediate32, VectorCompareFnImmediate64,
    VectorCompareFnRelative16, VectorCompareFnRelative32, VectorCompareFnRelative64,
};
use crate::structures::scanning::constraints::scan_constraint::ScanConstraint;

type PrimitiveType = u64;

const BYTE_COUNT_64: usize = 64;
const ELEMENT_COUNT_64: usize = BYTE_COUNT_64 / size_of::<PrimitiveType>();

const BYTE_COUNT_32: usize = 32;
const ELEMENT_COUNT_32: usize = BYTE_COUNT_64 / size_of::<PrimitiveType>();

const BYTE_COUNT_16: usize = 16;
const ELEMENT_COUNT_16: usize = BYTE_COUNT_64 / size_of::<PrimitiveType>();

impl VectorComparable for DataTypeU64be {
    fn get_vector_compare_equal_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_equal::<{ BYTE_COUNT_64 }, { ELEMENT_COUNT_64 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_equal_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_equal::<{ BYTE_COUNT_32 }, { ELEMENT_COUNT_32 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_equal_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_equal::<{ BYTE_COUNT_16 }, { ELEMENT_COUNT_16 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_not_equal_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_not_equal::<{ BYTE_COUNT_64 }, { ELEMENT_COUNT_64 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_not_equal_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_not_equal::<{ BYTE_COUNT_32 }, { ELEMENT_COUNT_32 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_not_equal_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_not_equal::<{ BYTE_COUNT_16 }, { ELEMENT_COUNT_16 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_greater_than_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_greater_than_unsigned::<{ BYTE_COUNT_64 }, { ELEMENT_COUNT_64 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_greater_than_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_greater_than_unsigned::<{ BYTE_COUNT_32 }, { ELEMENT_COUNT_32 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_greater_than_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_greater_than_unsigned::<{ BYTE_COUNT_16 }, { ELEMENT_COUNT_16 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_greater_than_or_equal_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_greater_than_or_equal_unsigned::<{ BYTE_COUNT_64 }, { ELEMENT_COUNT_64 }, PrimitiveType>(
            scan_constraint,
        )
    }

    fn get_vector_compare_greater_than_or_equal_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_greater_than_or_equal_unsigned::<{ BYTE_COUNT_32 }, { ELEMENT_COUNT_32 }, PrimitiveType>(
            scan_constraint,
        )
    }

    fn get_vector_compare_greater_than_or_equal_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_greater_than_or_equal_unsigned::<{ BYTE_COUNT_16 }, { ELEMENT_COUNT_16 }, PrimitiveType>(
            scan_constraint,
        )
    }

    fn get_vector_compare_less_than_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_less_than_unsigned::<{ BYTE_COUNT_64 }, { ELEMENT_COUNT_64 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_less_than_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_less_than_unsigned::<{ BYTE_COUNT_32 }, { ELEMENT_COUNT_32 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_less_than_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_less_than_unsigned::<{ BYTE_COUNT_16 }, { ELEMENT_COUNT_16 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_less_than_or_equal_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_less_than_or_equal_unsigned::<{ BYTE_COUNT_64 }, { ELEMENT_COUNT_64 }, PrimitiveType>(
            scan_constraint,
        )
    }

    fn get_vector_compare_less_than_or_equal_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_less_than_or_equal_unsigned::<{ BYTE_COUNT_32 }, { ELEMENT_COUNT_32 }, PrimitiveType>(
            scan_constraint,
        )
    }

    fn get_vector_compare_less_than_or_equal_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_less_than_or_equal_unsigned::<{ BYTE_COUNT_16 }, { ELEMENT_COUNT_16 }, PrimitiveType>(
            scan_constraint,
        )
    }

    fn get_vector_compare_changed_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_changed::<{ BYTE_COUNT_64 }, { ELEMENT_COUNT_64 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_changed_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_changed::<{ BYTE_COUNT_32 }, { ELEMENT_COUNT_32 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_changed_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_changed::<{ BYTE_COUNT_16 }, { ELEMENT_COUNT_16 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_unchanged_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_unchanged::<{ BYTE_COUNT_64 }, { ELEMENT_COUNT_64 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_unchanged_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_unchanged::<{ BYTE_COUNT_32 }, { ELEMENT_COUNT_32 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_unchanged_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_unchanged::<{ BYTE_COUNT_16 }, { ELEMENT_COUNT_16 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_increased_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_increased_unsigned::<{ BYTE_COUNT_64 }, { ELEMENT_COUNT_64 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_increased_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_increased_unsigned::<{ BYTE_COUNT_32 }, { ELEMENT_COUNT_32 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_increased_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_increased_unsigned::<{ BYTE_COUNT_16 }, { ELEMENT_COUNT_16 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_decreased_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_decreased_unsigned::<{ BYTE_COUNT_64 }, { ELEMENT_COUNT_64 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_decreased_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_decreased_unsigned::<{ BYTE_COUNT_32 }, { ELEMENT_COUNT_32 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_decreased_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_decreased_unsigned::<{ BYTE_COUNT_16 }, { ELEMENT_COUNT_16 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_increased_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_increased_by_unsigned::<{ BYTE_COUNT_64 }, { ELEMENT_COUNT_64 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_increased_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_increased_by_unsigned::<{ BYTE_COUNT_32 }, { ELEMENT_COUNT_32 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_increased_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_increased_by_unsigned::<{ BYTE_COUNT_16 }, { ELEMENT_COUNT_16 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_decreased_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_decreased_by_unsigned::<{ BYTE_COUNT_64 }, { ELEMENT_COUNT_64 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_decreased_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_decreased_by_unsigned::<{ BYTE_COUNT_32 }, { ELEMENT_COUNT_32 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_decreased_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_decreased_by_unsigned::<{ BYTE_COUNT_16 }, { ELEMENT_COUNT_16 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_multiplied_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_multiplied_by_unsigned::<{ BYTE_COUNT_64 }, { ELEMENT_COUNT_64 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_multiplied_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_multiplied_by_unsigned::<{ BYTE_COUNT_32 }, { ELEMENT_COUNT_32 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_multiplied_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_multiplied_by_unsigned::<{ BYTE_COUNT_16 }, { ELEMENT_COUNT_16 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_divided_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_divided_by_unsigned::<{ BYTE_COUNT_64 }, { ELEMENT_COUNT_64 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_divided_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_divided_by_unsigned::<{ BYTE_COUNT_32 }, { ELEMENT_COUNT_32 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_divided_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_divided_by_unsigned::<{ BYTE_COUNT_16 }, { ELEMENT_COUNT_16 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_modulo_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_modulo_by_unsigned::<{ BYTE_COUNT_64 }, { ELEMENT_COUNT_64 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_modulo_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_modulo_by_unsigned::<{ BYTE_COUNT_32 }, { ELEMENT_COUNT_32 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_modulo_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_modulo_by_unsigned::<{ BYTE_COUNT_16 }, { ELEMENT_COUNT_16 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_shift_left_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_shift_left_by_unsigned::<{ BYTE_COUNT_64 }, { ELEMENT_COUNT_64 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_shift_left_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_shift_left_by_unsigned::<{ BYTE_COUNT_32 }, { ELEMENT_COUNT_32 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_shift_left_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_shift_left_by_unsigned::<{ BYTE_COUNT_16 }, { ELEMENT_COUNT_16 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_shift_right_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_shift_right_by_unsigned::<{ BYTE_COUNT_64 }, { ELEMENT_COUNT_64 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_shift_right_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_shift_right_by_unsigned::<{ BYTE_COUNT_32 }, { ELEMENT_COUNT_32 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_shift_right_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_shift_right_by_unsigned::<{ BYTE_COUNT_16 }, { ELEMENT_COUNT_16 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_logical_and_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_logical_and_by_unsigned::<{ BYTE_COUNT_64 }, { ELEMENT_COUNT_64 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_logical_and_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_logical_and_by_unsigned::<{ BYTE_COUNT_32 }, { ELEMENT_COUNT_32 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_logical_and_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_logical_and_by_unsigned::<{ BYTE_COUNT_16 }, { ELEMENT_COUNT_16 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_logical_or_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_logical_or_by_unsigned::<{ BYTE_COUNT_64 }, { ELEMENT_COUNT_64 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_logical_or_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_logical_or_by_unsigned::<{ BYTE_COUNT_32 }, { ELEMENT_COUNT_32 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_logical_or_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_logical_or_by_unsigned::<{ BYTE_COUNT_16 }, { ELEMENT_COUNT_16 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_logical_xor_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_logical_xor_by_unsigned::<{ BYTE_COUNT_64 }, { ELEMENT_COUNT_64 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_logical_xor_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_logical_xor_by_unsigned::<{ BYTE_COUNT_32 }, { ELEMENT_COUNT_32 }, PrimitiveType>(scan_constraint)
    }

    fn get_vector_compare_logical_xor_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_logical_xor_by_unsigned::<{ BYTE_COUNT_16 }, { ELEMENT_COUNT_16 }, PrimitiveType>(scan_constraint)
    }
}
