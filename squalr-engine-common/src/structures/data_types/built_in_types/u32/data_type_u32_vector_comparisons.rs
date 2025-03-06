use crate::structures::data_types::built_in_types::u32::data_type_u32::DataTypeU32;
use crate::structures::data_types::comparisons::vector_comparable::VectorComparable;
use crate::structures::data_types::comparisons::vector_comparable::{
    VectorCompareFnDelta16, VectorCompareFnDelta32, VectorCompareFnDelta64, VectorCompareFnImmediate16, VectorCompareFnImmediate32, VectorCompareFnImmediate64,
    VectorCompareFnRelative16, VectorCompareFnRelative32, VectorCompareFnRelative64,
};
use crate::structures::data_types::comparisons::vector_comparisons_integer::VectorComparisonsInteger;
use crate::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use crate::structures::scanning::scan_parameters_local::ScanParametersLocal;

type PrimitiveType = u32;

impl VectorComparable for DataTypeU32 {
    fn get_vector_compare_equal_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        VectorComparisonsInteger::get_vector_compare_equal::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_equal_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        VectorComparisonsInteger::get_vector_compare_equal::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_equal_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        VectorComparisonsInteger::get_vector_compare_equal::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_not_equal_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        VectorComparisonsInteger::get_vector_compare_not_equal::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_not_equal_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        VectorComparisonsInteger::get_vector_compare_not_equal::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_not_equal_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        VectorComparisonsInteger::get_vector_compare_not_equal::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_greater_than_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        VectorComparisonsInteger::get_vector_compare_greater_than::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_greater_than_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        VectorComparisonsInteger::get_vector_compare_greater_than::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_greater_than_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        VectorComparisonsInteger::get_vector_compare_greater_than::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_greater_than_or_equal_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        VectorComparisonsInteger::get_vector_compare_greater_than_or_equal::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_greater_than_or_equal_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        VectorComparisonsInteger::get_vector_compare_greater_than_or_equal::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_greater_than_or_equal_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        VectorComparisonsInteger::get_vector_compare_greater_than_or_equal::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_less_than_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        VectorComparisonsInteger::get_vector_compare_less_than::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_less_than_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        VectorComparisonsInteger::get_vector_compare_less_than::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_less_than_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        VectorComparisonsInteger::get_vector_compare_less_than::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_less_than_or_equal_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        VectorComparisonsInteger::get_vector_compare_less_than_or_equal::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_less_than_or_equal_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        VectorComparisonsInteger::get_vector_compare_less_than_or_equal::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_less_than_or_equal_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        VectorComparisonsInteger::get_vector_compare_less_than_or_equal::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_changed_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative64> {
        VectorComparisonsInteger::get_vector_compare_changed::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_changed_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative32> {
        VectorComparisonsInteger::get_vector_compare_changed::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_changed_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative16> {
        VectorComparisonsInteger::get_vector_compare_changed::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_unchanged_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative64> {
        VectorComparisonsInteger::get_vector_compare_unchanged::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_unchanged_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative32> {
        VectorComparisonsInteger::get_vector_compare_unchanged::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_unchanged_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative16> {
        VectorComparisonsInteger::get_vector_compare_unchanged::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_increased_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative64> {
        VectorComparisonsInteger::get_vector_compare_increased::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_increased_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative32> {
        VectorComparisonsInteger::get_vector_compare_increased::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_increased_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative16> {
        VectorComparisonsInteger::get_vector_compare_increased::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_decreased_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative64> {
        VectorComparisonsInteger::get_vector_compare_decreased::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_decreased_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative32> {
        VectorComparisonsInteger::get_vector_compare_decreased::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_decreased_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative16> {
        VectorComparisonsInteger::get_vector_compare_decreased::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_increased_by_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta64> {
        VectorComparisonsInteger::get_vector_compare_increased_by::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_increased_by_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta32> {
        VectorComparisonsInteger::get_vector_compare_increased_by::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_increased_by_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta16> {
        VectorComparisonsInteger::get_vector_compare_increased_by::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_decreased_by_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta64> {
        VectorComparisonsInteger::get_vector_compare_decreased_by::<64, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_decreased_by_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta32> {
        VectorComparisonsInteger::get_vector_compare_decreased_by::<32, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_decreased_by_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta16> {
        VectorComparisonsInteger::get_vector_compare_decreased_by::<16, PrimitiveType>(scan_parameters_global, scan_parameters_local)
    }
}
