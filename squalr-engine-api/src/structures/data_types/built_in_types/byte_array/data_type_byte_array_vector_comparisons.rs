use crate::structures::data_types::built_in_types::byte_array::data_type_byte_array::DataTypeByteArray;
use crate::structures::data_types::comparisons::vector_comparable::VectorComparable;
use crate::structures::data_types::comparisons::vector_comparable::{
    VectorCompareFnDelta16, VectorCompareFnDelta32, VectorCompareFnDelta64, VectorCompareFnImmediate16, VectorCompareFnImmediate32, VectorCompareFnImmediate64,
    VectorCompareFnRelative16, VectorCompareFnRelative32, VectorCompareFnRelative64,
};
use crate::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use crate::structures::scanning::scan_parameters_local::ScanParametersLocal;

/// Deliberately not implemented. Vector based byte array comparisons are implemented elsewhere in specialized scan routines.
impl VectorComparable for DataTypeByteArray {
    fn get_vector_compare_equal_64(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        None
    }

    fn get_vector_compare_equal_32(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        None
    }

    fn get_vector_compare_equal_16(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        None
    }

    fn get_vector_compare_not_equal_64(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        None
    }

    fn get_vector_compare_not_equal_32(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        None
    }

    fn get_vector_compare_not_equal_16(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        None
    }

    fn get_vector_compare_greater_than_64(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        None
    }

    fn get_vector_compare_greater_than_32(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        None
    }

    fn get_vector_compare_greater_than_16(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        None
    }

    fn get_vector_compare_greater_than_or_equal_64(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        None
    }

    fn get_vector_compare_greater_than_or_equal_32(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        None
    }

    fn get_vector_compare_greater_than_or_equal_16(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        None
    }

    fn get_vector_compare_less_than_64(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        None
    }

    fn get_vector_compare_less_than_32(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        None
    }

    fn get_vector_compare_less_than_16(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        None
    }

    fn get_vector_compare_less_than_or_equal_64(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        None
    }

    fn get_vector_compare_less_than_or_equal_32(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        None
    }

    fn get_vector_compare_less_than_or_equal_16(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        None
    }

    fn get_vector_compare_changed_64(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative64> {
        None
    }

    fn get_vector_compare_changed_32(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative32> {
        None
    }

    fn get_vector_compare_changed_16(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative16> {
        None
    }

    fn get_vector_compare_unchanged_64(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative64> {
        None
    }

    fn get_vector_compare_unchanged_32(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative32> {
        None
    }

    fn get_vector_compare_unchanged_16(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative16> {
        None
    }

    fn get_vector_compare_increased_64(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative64> {
        None
    }

    fn get_vector_compare_increased_32(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative32> {
        None
    }

    fn get_vector_compare_increased_16(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative16> {
        None
    }

    fn get_vector_compare_decreased_64(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative64> {
        None
    }

    fn get_vector_compare_decreased_32(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative32> {
        None
    }

    fn get_vector_compare_decreased_16(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative16> {
        None
    }

    fn get_vector_compare_increased_by_64(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta64> {
        None
    }

    fn get_vector_compare_increased_by_32(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta32> {
        None
    }

    fn get_vector_compare_increased_by_16(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta16> {
        None
    }

    fn get_vector_compare_decreased_by_64(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta64> {
        None
    }

    fn get_vector_compare_decreased_by_32(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta32> {
        None
    }

    fn get_vector_compare_decreased_by_16(
        &self,
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta16> {
        None
    }
}
