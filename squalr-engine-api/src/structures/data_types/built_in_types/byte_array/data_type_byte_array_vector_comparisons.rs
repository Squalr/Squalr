use crate::structures::data_types::built_in_types::byte_array::data_type_byte_array::DataTypeByteArray;
use crate::structures::data_types::comparisons::vector_comparable::VectorComparable;
use crate::structures::scanning::comparisons::scan_function_vector::{
    VectorCompareFnDelta16, VectorCompareFnDelta32, VectorCompareFnDelta64, VectorCompareFnImmediate16, VectorCompareFnImmediate32, VectorCompareFnImmediate64,
    VectorCompareFnRelative16, VectorCompareFnRelative32, VectorCompareFnRelative64,
};
use crate::structures::scanning::parameters::mapped::mapped_scan_parameters::MappedScanParameters;

/// Deliberately not implemented. Vector based byte array comparisons are implemented elsewhere in specialized scan routines.
impl VectorComparable for DataTypeByteArray {
    fn get_vector_compare_equal_64(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate64> {
        None
    }

    fn get_vector_compare_equal_32(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate32> {
        None
    }

    fn get_vector_compare_equal_16(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate16> {
        None
    }

    fn get_vector_compare_not_equal_64(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate64> {
        None
    }

    fn get_vector_compare_not_equal_32(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate32> {
        None
    }

    fn get_vector_compare_not_equal_16(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate16> {
        None
    }

    fn get_vector_compare_greater_than_64(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate64> {
        None
    }

    fn get_vector_compare_greater_than_32(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate32> {
        None
    }

    fn get_vector_compare_greater_than_16(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate16> {
        None
    }

    fn get_vector_compare_greater_than_or_equal_64(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate64> {
        None
    }

    fn get_vector_compare_greater_than_or_equal_32(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate32> {
        None
    }

    fn get_vector_compare_greater_than_or_equal_16(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate16> {
        None
    }

    fn get_vector_compare_less_than_64(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate64> {
        None
    }

    fn get_vector_compare_less_than_32(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate32> {
        None
    }

    fn get_vector_compare_less_than_16(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate16> {
        None
    }

    fn get_vector_compare_less_than_or_equal_64(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate64> {
        None
    }

    fn get_vector_compare_less_than_or_equal_32(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate32> {
        None
    }

    fn get_vector_compare_less_than_or_equal_16(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate16> {
        None
    }

    fn get_vector_compare_changed_64(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative64> {
        None
    }

    fn get_vector_compare_changed_32(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative32> {
        None
    }

    fn get_vector_compare_changed_16(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative16> {
        None
    }

    fn get_vector_compare_unchanged_64(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative64> {
        None
    }

    fn get_vector_compare_unchanged_32(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative32> {
        None
    }

    fn get_vector_compare_unchanged_16(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative16> {
        None
    }

    fn get_vector_compare_increased_64(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative64> {
        None
    }

    fn get_vector_compare_increased_32(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative32> {
        None
    }

    fn get_vector_compare_increased_16(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative16> {
        None
    }

    fn get_vector_compare_decreased_64(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative64> {
        None
    }

    fn get_vector_compare_decreased_32(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative32> {
        None
    }

    fn get_vector_compare_decreased_16(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative16> {
        None
    }

    fn get_vector_compare_increased_by_64(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta64> {
        None
    }

    fn get_vector_compare_increased_by_32(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta32> {
        None
    }

    fn get_vector_compare_increased_by_16(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta16> {
        None
    }

    fn get_vector_compare_decreased_by_64(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta64> {
        None
    }

    fn get_vector_compare_decreased_by_32(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta32> {
        None
    }

    fn get_vector_compare_decreased_by_16(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta16> {
        None
    }

    fn get_vector_compare_multiplied_by_64(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta64> {
        None
    }

    fn get_vector_compare_multiplied_by_32(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta32> {
        None
    }

    fn get_vector_compare_multiplied_by_16(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta16> {
        None
    }

    fn get_vector_compare_divided_by_64(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta64> {
        None
    }

    fn get_vector_compare_divided_by_32(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta32> {
        None
    }

    fn get_vector_compare_divided_by_16(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta16> {
        None
    }

    fn get_vector_compare_modulo_by_64(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta64> {
        None
    }

    fn get_vector_compare_modulo_by_32(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta32> {
        None
    }

    fn get_vector_compare_modulo_by_16(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta16> {
        None
    }

    fn get_vector_compare_shift_left_by_64(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta64> {
        None
    }

    fn get_vector_compare_shift_left_by_32(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta32> {
        None
    }

    fn get_vector_compare_shift_left_by_16(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta16> {
        None
    }
    fn get_vector_compare_shift_right_by_64(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta64> {
        None
    }

    fn get_vector_compare_shift_right_by_32(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta32> {
        None
    }

    fn get_vector_compare_shift_right_by_16(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta16> {
        None
    }

    fn get_vector_compare_logical_and_by_64(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta64> {
        None
    }

    fn get_vector_compare_logical_and_by_32(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta32> {
        None
    }

    fn get_vector_compare_logical_and_by_16(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta16> {
        None
    }
    fn get_vector_compare_logical_or_by_64(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta64> {
        None
    }

    fn get_vector_compare_logical_or_by_32(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta32> {
        None
    }

    fn get_vector_compare_logical_or_by_16(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta16> {
        None
    }
    fn get_vector_compare_logical_xor_by_64(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta64> {
        None
    }

    fn get_vector_compare_logical_xor_by_32(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta32> {
        None
    }

    fn get_vector_compare_logical_xor_by_16(
        &self,
        _scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta16> {
        None
    }
}
