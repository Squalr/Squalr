use crate::structures::data_types::built_in_types::u32be::data_type_u32be::DataTypeU32be;
use crate::structures::data_types::comparisons::vector_comparable::VectorComparable;
use crate::structures::data_types::comparisons::vector_comparable::{
    VectorCompareFnDelta16, VectorCompareFnDelta32, VectorCompareFnDelta64, VectorCompareFnImmediate16, VectorCompareFnImmediate32, VectorCompareFnImmediate64,
    VectorCompareFnRelative16, VectorCompareFnRelative32, VectorCompareFnRelative64,
};
use crate::structures::data_types::comparisons::vector_comparisons_integer_big_endian::VectorComparisonsIntegerBigEndian;
use crate::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use crate::structures::scanning::scan_parameters_local::ScanParametersLocal;

type PrimitiveType = u32;

impl VectorComparable for DataTypeU32be {
    fn get_vector_compare_equal_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_equal::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_equal_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_equal::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_equal_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_equal::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_not_equal_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_not_equal::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_not_equal_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_not_equal::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_not_equal_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_not_equal::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_greater_than_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_greater_than_unsigned::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_greater_than_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_greater_than_unsigned::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_greater_than_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_greater_than_unsigned::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_greater_than_or_equal_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_greater_than_or_equal_unsigned::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_greater_than_or_equal_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_greater_than_or_equal_unsigned::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_greater_than_or_equal_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_greater_than_or_equal_unsigned::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_less_than_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_less_than_unsigned::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_less_than_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_less_than_unsigned::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_less_than_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_less_than_unsigned::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_less_than_or_equal_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_less_than_or_equal_unsigned::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_less_than_or_equal_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_less_than_or_equal_unsigned::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_less_than_or_equal_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_less_than_or_equal_unsigned::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_changed_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_changed::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_changed_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_changed::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_changed_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_changed::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_unchanged_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_unchanged::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_unchanged_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_unchanged::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_unchanged_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_unchanged::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_increased_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_increased_unsigned::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_increased_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_increased_unsigned::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_increased_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_increased_unsigned::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_decreased_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_decreased_unsigned::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_decreased_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_decreased_unsigned::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_decreased_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_decreased_unsigned::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_increased_by_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_increased_by_unsigned::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_increased_by_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_increased_by_unsigned::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_increased_by_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_increased_by_unsigned::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_decreased_by_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta64> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_decreased_by_unsigned::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_decreased_by_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta32> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_decreased_by_unsigned::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_decreased_by_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta16> {
        VectorComparisonsIntegerBigEndian::get_vector_compare_decreased_by_unsigned::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }
}
