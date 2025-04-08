use crate::structures::scanning::comparisons::scan_compare_type_delta::ScanCompareTypeDelta;
use crate::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use crate::structures::scanning::comparisons::scan_compare_type_relative::ScanCompareTypeRelative;
use crate::structures::scanning::comparisons::scan_function_vector::{
    VectorCompareFnDelta16, VectorCompareFnDelta32, VectorCompareFnDelta64, VectorCompareFnImmediate16, VectorCompareFnImmediate32, VectorCompareFnImmediate64,
    VectorCompareFnRelative16, VectorCompareFnRelative32, VectorCompareFnRelative64,
};
use crate::structures::scanning::parameters::mapped::mapped_scan_parameters::MappedScanParameters;

pub trait VectorComparable {
    fn get_vector_compare_equal_64(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate64>;
    fn get_vector_compare_equal_32(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate32>;
    fn get_vector_compare_equal_16(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate16>;
    fn get_vector_compare_not_equal_64(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate64>;
    fn get_vector_compare_not_equal_32(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate32>;
    fn get_vector_compare_not_equal_16(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate16>;
    fn get_vector_compare_greater_than_64(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate64>;
    fn get_vector_compare_greater_than_32(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate32>;
    fn get_vector_compare_greater_than_16(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate16>;
    fn get_vector_compare_greater_than_or_equal_64(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate64>;
    fn get_vector_compare_greater_than_or_equal_32(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate32>;
    fn get_vector_compare_greater_than_or_equal_16(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate16>;
    fn get_vector_compare_less_than_64(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate64>;
    fn get_vector_compare_less_than_32(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate32>;
    fn get_vector_compare_less_than_16(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate16>;
    fn get_vector_compare_less_than_or_equal_64(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate64>;
    fn get_vector_compare_less_than_or_equal_32(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate32>;
    fn get_vector_compare_less_than_or_equal_16(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate16>;

    fn get_vector_compare_changed_64(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative64>;
    fn get_vector_compare_changed_32(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative32>;
    fn get_vector_compare_changed_16(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative16>;
    fn get_vector_compare_unchanged_64(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative64>;
    fn get_vector_compare_unchanged_32(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative32>;
    fn get_vector_compare_unchanged_16(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative16>;
    fn get_vector_compare_increased_64(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative64>;
    fn get_vector_compare_increased_32(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative32>;
    fn get_vector_compare_increased_16(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative16>;
    fn get_vector_compare_decreased_64(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative64>;
    fn get_vector_compare_decreased_32(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative32>;
    fn get_vector_compare_decreased_16(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative16>;

    fn get_vector_compare_increased_by_64(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta64>;
    fn get_vector_compare_increased_by_32(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta32>;
    fn get_vector_compare_increased_by_16(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta16>;
    fn get_vector_compare_decreased_by_64(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta64>;
    fn get_vector_compare_decreased_by_32(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta32>;
    fn get_vector_compare_decreased_by_16(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta16>;
    fn get_vector_compare_multiplied_by_64(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta64>;
    fn get_vector_compare_multiplied_by_32(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta32>;
    fn get_vector_compare_multiplied_by_16(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta16>;
    fn get_vector_compare_divided_by_64(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta64>;
    fn get_vector_compare_divided_by_32(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta32>;
    fn get_vector_compare_divided_by_16(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta16>;
    fn get_vector_compare_modulo_by_64(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta64>;
    fn get_vector_compare_modulo_by_32(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta32>;
    fn get_vector_compare_modulo_by_16(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta16>;
    fn get_vector_compare_shift_left_by_64(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta64>;
    fn get_vector_compare_shift_left_by_32(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta32>;
    fn get_vector_compare_shift_left_by_16(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta16>;
    fn get_vector_compare_shift_right_by_64(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta64>;
    fn get_vector_compare_shift_right_by_32(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta32>;
    fn get_vector_compare_shift_right_by_16(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta16>;
    fn get_vector_compare_logical_and_by_64(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta64>;
    fn get_vector_compare_logical_and_by_32(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta32>;
    fn get_vector_compare_logical_and_by_16(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta16>;
    fn get_vector_compare_logical_or_by_64(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta64>;
    fn get_vector_compare_logical_or_by_32(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta32>;
    fn get_vector_compare_logical_or_by_16(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta16>;
    fn get_vector_compare_logical_xor_by_64(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta64>;
    fn get_vector_compare_logical_xor_by_32(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta32>;
    fn get_vector_compare_logical_xor_by_16(
        &self,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta16>;

    fn get_vector_compare_func_immediate_64(
        &self,
        scan_compare_type: &ScanCompareTypeImmediate,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate64> {
        match scan_compare_type {
            ScanCompareTypeImmediate::Equal => self.get_vector_compare_equal_64(scan_parameters),
            ScanCompareTypeImmediate::NotEqual => self.get_vector_compare_not_equal_64(scan_parameters),
            ScanCompareTypeImmediate::GreaterThan => self.get_vector_compare_greater_than_64(scan_parameters),
            ScanCompareTypeImmediate::GreaterThanOrEqual => self.get_vector_compare_greater_than_or_equal_64(scan_parameters),
            ScanCompareTypeImmediate::LessThan => self.get_vector_compare_less_than_64(scan_parameters),
            ScanCompareTypeImmediate::LessThanOrEqual => self.get_vector_compare_less_than_or_equal_64(scan_parameters),
        }
    }

    fn get_vector_compare_func_immediate_32(
        &self,
        scan_compare_type: &ScanCompareTypeImmediate,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate32> {
        match scan_compare_type {
            ScanCompareTypeImmediate::Equal => self.get_vector_compare_equal_32(scan_parameters),
            ScanCompareTypeImmediate::NotEqual => self.get_vector_compare_not_equal_32(scan_parameters),
            ScanCompareTypeImmediate::GreaterThan => self.get_vector_compare_greater_than_32(scan_parameters),
            ScanCompareTypeImmediate::GreaterThanOrEqual => self.get_vector_compare_greater_than_or_equal_32(scan_parameters),
            ScanCompareTypeImmediate::LessThan => self.get_vector_compare_less_than_32(scan_parameters),
            ScanCompareTypeImmediate::LessThanOrEqual => self.get_vector_compare_less_than_or_equal_32(scan_parameters),
        }
    }

    fn get_vector_compare_func_immediate_16(
        &self,
        scan_compare_type: &ScanCompareTypeImmediate,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate16> {
        match scan_compare_type {
            ScanCompareTypeImmediate::Equal => self.get_vector_compare_equal_16(scan_parameters),
            ScanCompareTypeImmediate::NotEqual => self.get_vector_compare_not_equal_16(scan_parameters),
            ScanCompareTypeImmediate::GreaterThan => self.get_vector_compare_greater_than_16(scan_parameters),
            ScanCompareTypeImmediate::GreaterThanOrEqual => self.get_vector_compare_greater_than_or_equal_16(scan_parameters),
            ScanCompareTypeImmediate::LessThan => self.get_vector_compare_less_than_16(scan_parameters),
            ScanCompareTypeImmediate::LessThanOrEqual => self.get_vector_compare_less_than_or_equal_16(scan_parameters),
        }
    }

    fn get_vector_compare_func_relative_64(
        &self,
        scan_compare_type: &ScanCompareTypeRelative,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative64> {
        match scan_compare_type {
            ScanCompareTypeRelative::Changed => self.get_vector_compare_changed_64(scan_parameters),
            ScanCompareTypeRelative::Unchanged => self.get_vector_compare_unchanged_64(scan_parameters),
            ScanCompareTypeRelative::Increased => self.get_vector_compare_increased_64(scan_parameters),
            ScanCompareTypeRelative::Decreased => self.get_vector_compare_decreased_64(scan_parameters),
        }
    }

    fn get_vector_compare_func_relative_32(
        &self,
        scan_compare_type: &ScanCompareTypeRelative,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative32> {
        match scan_compare_type {
            ScanCompareTypeRelative::Changed => self.get_vector_compare_changed_32(scan_parameters),
            ScanCompareTypeRelative::Unchanged => self.get_vector_compare_unchanged_32(scan_parameters),
            ScanCompareTypeRelative::Increased => self.get_vector_compare_increased_32(scan_parameters),
            ScanCompareTypeRelative::Decreased => self.get_vector_compare_decreased_32(scan_parameters),
        }
    }

    fn get_vector_compare_func_relative_16(
        &self,
        scan_compare_type: &ScanCompareTypeRelative,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative16> {
        match scan_compare_type {
            ScanCompareTypeRelative::Changed => self.get_vector_compare_changed_16(scan_parameters),
            ScanCompareTypeRelative::Unchanged => self.get_vector_compare_unchanged_16(scan_parameters),
            ScanCompareTypeRelative::Increased => self.get_vector_compare_increased_16(scan_parameters),
            ScanCompareTypeRelative::Decreased => self.get_vector_compare_decreased_16(scan_parameters),
        }
    }

    fn get_vector_compare_func_delta_64(
        &self,
        scan_compare_type: &ScanCompareTypeDelta,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta64> {
        match scan_compare_type {
            ScanCompareTypeDelta::IncreasedByX => self.get_vector_compare_increased_by_64(scan_parameters),
            ScanCompareTypeDelta::DecreasedByX => self.get_vector_compare_decreased_by_64(scan_parameters),
            ScanCompareTypeDelta::MultipliedByX => self.get_vector_compare_multiplied_by_64(scan_parameters),
            ScanCompareTypeDelta::DividedByX => self.get_vector_compare_divided_by_64(scan_parameters),
            ScanCompareTypeDelta::ModuloByX => self.get_vector_compare_modulo_by_64(scan_parameters),
            ScanCompareTypeDelta::ShiftLeftByX => self.get_vector_compare_shift_left_by_64(scan_parameters),
            ScanCompareTypeDelta::ShiftRightByX => self.get_vector_compare_shift_right_by_64(scan_parameters),
            ScanCompareTypeDelta::LogicalAndByX => self.get_vector_compare_logical_and_by_64(scan_parameters),
            ScanCompareTypeDelta::LogicalOrByX => self.get_vector_compare_logical_or_by_64(scan_parameters),
            ScanCompareTypeDelta::LogicalXorByX => self.get_vector_compare_logical_xor_by_64(scan_parameters),
        }
    }

    fn get_vector_compare_func_delta_32(
        &self,
        scan_compare_type: &ScanCompareTypeDelta,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta32> {
        match scan_compare_type {
            ScanCompareTypeDelta::IncreasedByX => self.get_vector_compare_increased_by_32(scan_parameters),
            ScanCompareTypeDelta::DecreasedByX => self.get_vector_compare_decreased_by_32(scan_parameters),
            ScanCompareTypeDelta::MultipliedByX => self.get_vector_compare_multiplied_by_32(scan_parameters),
            ScanCompareTypeDelta::DividedByX => self.get_vector_compare_divided_by_32(scan_parameters),
            ScanCompareTypeDelta::ModuloByX => self.get_vector_compare_modulo_by_32(scan_parameters),
            ScanCompareTypeDelta::ShiftLeftByX => self.get_vector_compare_shift_left_by_32(scan_parameters),
            ScanCompareTypeDelta::ShiftRightByX => self.get_vector_compare_shift_right_by_32(scan_parameters),
            ScanCompareTypeDelta::LogicalAndByX => self.get_vector_compare_logical_and_by_32(scan_parameters),
            ScanCompareTypeDelta::LogicalOrByX => self.get_vector_compare_logical_or_by_32(scan_parameters),
            ScanCompareTypeDelta::LogicalXorByX => self.get_vector_compare_logical_xor_by_32(scan_parameters),
        }
    }

    fn get_vector_compare_func_delta_16(
        &self,
        scan_compare_type: &ScanCompareTypeDelta,
        scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta16> {
        match scan_compare_type {
            ScanCompareTypeDelta::IncreasedByX => self.get_vector_compare_increased_by_16(scan_parameters),
            ScanCompareTypeDelta::DecreasedByX => self.get_vector_compare_decreased_by_16(scan_parameters),
            ScanCompareTypeDelta::MultipliedByX => self.get_vector_compare_multiplied_by_16(scan_parameters),
            ScanCompareTypeDelta::DividedByX => self.get_vector_compare_divided_by_16(scan_parameters),
            ScanCompareTypeDelta::ModuloByX => self.get_vector_compare_modulo_by_16(scan_parameters),
            ScanCompareTypeDelta::ShiftLeftByX => self.get_vector_compare_shift_left_by_16(scan_parameters),
            ScanCompareTypeDelta::ShiftRightByX => self.get_vector_compare_shift_right_by_16(scan_parameters),
            ScanCompareTypeDelta::LogicalAndByX => self.get_vector_compare_logical_and_by_16(scan_parameters),
            ScanCompareTypeDelta::LogicalOrByX => self.get_vector_compare_logical_or_by_16(scan_parameters),
            ScanCompareTypeDelta::LogicalXorByX => self.get_vector_compare_logical_xor_by_16(scan_parameters),
        }
    }
}
