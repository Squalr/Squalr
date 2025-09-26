use crate::structures::data_types::built_in_types::f64be::data_type_f64be::DataTypeF64be;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarComparable;
use crate::structures::data_types::comparisons::scalar_comparisons_float_big_endian::ScalarComparisonsFloatBigEndian;
use crate::structures::scanning::comparisons::scan_function_scalar::{ScalarCompareFnDelta, ScalarCompareFnImmediate, ScalarCompareFnRelative};
use crate::structures::scanning::parameters::mapped::mapped_scan_parameters::MappedScanParameters;

type PrimitiveType = f64;

impl ScalarComparable for DataTypeF64be {
    fn get_compare_equal(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsFloatBigEndian::get_compare_equal::<PrimitiveType>(mapped_scan_parameters)
    }

    fn get_compare_not_equal(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsFloatBigEndian::get_compare_not_equal::<PrimitiveType>(mapped_scan_parameters)
    }

    fn get_compare_greater_than(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsFloatBigEndian::get_compare_greater_than::<PrimitiveType>(mapped_scan_parameters)
    }

    fn get_compare_greater_than_or_equal(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsFloatBigEndian::get_compare_greater_than_or_equal::<PrimitiveType>(mapped_scan_parameters)
    }

    fn get_compare_less_than(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsFloatBigEndian::get_compare_less_than::<PrimitiveType>(mapped_scan_parameters)
    }

    fn get_compare_less_than_or_equal(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        ScalarComparisonsFloatBigEndian::get_compare_less_than_or_equal::<PrimitiveType>(mapped_scan_parameters)
    }

    fn get_compare_changed(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsFloatBigEndian::get_compare_changed::<PrimitiveType>(mapped_scan_parameters)
    }

    fn get_compare_unchanged(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsFloatBigEndian::get_compare_unchanged::<PrimitiveType>(mapped_scan_parameters)
    }

    fn get_compare_increased(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsFloatBigEndian::get_compare_increased::<PrimitiveType>(mapped_scan_parameters)
    }

    fn get_compare_decreased(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnRelative> {
        ScalarComparisonsFloatBigEndian::get_compare_decreased::<PrimitiveType>(mapped_scan_parameters)
    }

    fn get_compare_increased_by(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsFloatBigEndian::get_compare_increased_by::<PrimitiveType>(mapped_scan_parameters)
    }

    fn get_compare_decreased_by(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsFloatBigEndian::get_compare_decreased_by::<PrimitiveType>(mapped_scan_parameters)
    }

    fn get_compare_multiplied_by(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsFloatBigEndian::get_compare_multiplied_by::<PrimitiveType>(mapped_scan_parameters)
    }

    fn get_compare_divided_by(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsFloatBigEndian::get_compare_divided_by::<PrimitiveType>(mapped_scan_parameters)
    }

    fn get_compare_modulo_by(
        &self,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        ScalarComparisonsFloatBigEndian::get_compare_modulo_by::<PrimitiveType>(mapped_scan_parameters)
    }

    fn get_compare_shift_left_by(
        &self,
        _mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        None
    }

    fn get_compare_shift_right_by(
        &self,
        _mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        None
    }

    fn get_compare_logical_and_by(
        &self,
        _mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        None
    }

    fn get_compare_logical_or_by(
        &self,
        _mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        None
    }

    fn get_compare_logical_xor_by(
        &self,
        _mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnDelta> {
        None
    }
}
