use crate::structures::scanning::comparisons::scan_compare_type_delta::ScanCompareTypeDelta;
use crate::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use crate::structures::scanning::comparisons::scan_compare_type_relative::ScanCompareTypeRelative;
use crate::structures::scanning::comparisons::scan_function_vector::{
    VectorCompareFnDelta16, VectorCompareFnDelta32, VectorCompareFnDelta64, VectorCompareFnImmediate16, VectorCompareFnImmediate32, VectorCompareFnImmediate64,
    VectorCompareFnRelative16, VectorCompareFnRelative32, VectorCompareFnRelative64,
};
use crate::structures::scanning::constraints::scan_constraint::ScanConstraint;

pub trait VectorComparable {
    fn get_vector_compare_equal_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate64>;

    fn get_vector_compare_equal_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate32>;

    fn get_vector_compare_equal_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate16>;

    fn get_vector_compare_not_equal_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate64>;

    fn get_vector_compare_not_equal_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate32>;

    fn get_vector_compare_not_equal_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate16>;

    fn get_vector_compare_greater_than_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate64>;

    fn get_vector_compare_greater_than_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate32>;

    fn get_vector_compare_greater_than_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate16>;

    fn get_vector_compare_greater_than_or_equal_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate64>;

    fn get_vector_compare_greater_than_or_equal_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate32>;

    fn get_vector_compare_greater_than_or_equal_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate16>;

    fn get_vector_compare_less_than_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate64>;

    fn get_vector_compare_less_than_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate32>;

    fn get_vector_compare_less_than_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate16>;

    fn get_vector_compare_less_than_or_equal_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate64>;

    fn get_vector_compare_less_than_or_equal_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate32>;

    fn get_vector_compare_less_than_or_equal_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate16>;

    fn get_vector_compare_changed_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative64>;

    fn get_vector_compare_changed_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative32>;

    fn get_vector_compare_changed_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative16>;

    fn get_vector_compare_unchanged_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative64>;

    fn get_vector_compare_unchanged_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative32>;

    fn get_vector_compare_unchanged_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative16>;

    fn get_vector_compare_increased_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative64>;

    fn get_vector_compare_increased_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative32>;

    fn get_vector_compare_increased_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative16>;

    fn get_vector_compare_decreased_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative64>;

    fn get_vector_compare_decreased_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative32>;

    fn get_vector_compare_decreased_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative16>;

    fn get_vector_compare_increased_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64>;

    fn get_vector_compare_increased_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32>;

    fn get_vector_compare_increased_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16>;

    fn get_vector_compare_decreased_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64>;

    fn get_vector_compare_decreased_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32>;

    fn get_vector_compare_decreased_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16>;

    fn get_vector_compare_multiplied_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64>;

    fn get_vector_compare_multiplied_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32>;

    fn get_vector_compare_multiplied_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16>;

    fn get_vector_compare_divided_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64>;

    fn get_vector_compare_divided_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32>;

    fn get_vector_compare_divided_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16>;

    fn get_vector_compare_modulo_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64>;

    fn get_vector_compare_modulo_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32>;

    fn get_vector_compare_modulo_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16>;

    fn get_vector_compare_shift_left_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64>;

    fn get_vector_compare_shift_left_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32>;

    fn get_vector_compare_shift_left_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16>;

    fn get_vector_compare_shift_right_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64>;

    fn get_vector_compare_shift_right_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32>;

    fn get_vector_compare_shift_right_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16>;

    fn get_vector_compare_logical_and_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64>;

    fn get_vector_compare_logical_and_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32>;

    fn get_vector_compare_logical_and_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16>;

    fn get_vector_compare_logical_or_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64>;

    fn get_vector_compare_logical_or_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32>;

    fn get_vector_compare_logical_or_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16>;

    fn get_vector_compare_logical_xor_by_64(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64>;

    fn get_vector_compare_logical_xor_by_32(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32>;

    fn get_vector_compare_logical_xor_by_16(
        &self,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16>;

    fn get_vector_compare_func_immediate_64(
        &self,
        scan_compare_type: &ScanCompareTypeImmediate,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate64> {
        match scan_compare_type {
            ScanCompareTypeImmediate::Equal => self.get_vector_compare_equal_64(scan_constraint),
            ScanCompareTypeImmediate::NotEqual => self.get_vector_compare_not_equal_64(scan_constraint),
            ScanCompareTypeImmediate::GreaterThan => self.get_vector_compare_greater_than_64(scan_constraint),
            ScanCompareTypeImmediate::GreaterThanOrEqual => self.get_vector_compare_greater_than_or_equal_64(scan_constraint),
            ScanCompareTypeImmediate::LessThan => self.get_vector_compare_less_than_64(scan_constraint),
            ScanCompareTypeImmediate::LessThanOrEqual => self.get_vector_compare_less_than_or_equal_64(scan_constraint),
        }
    }

    fn get_vector_compare_func_immediate_32(
        &self,
        scan_compare_type: &ScanCompareTypeImmediate,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate32> {
        match scan_compare_type {
            ScanCompareTypeImmediate::Equal => self.get_vector_compare_equal_32(scan_constraint),
            ScanCompareTypeImmediate::NotEqual => self.get_vector_compare_not_equal_32(scan_constraint),
            ScanCompareTypeImmediate::GreaterThan => self.get_vector_compare_greater_than_32(scan_constraint),
            ScanCompareTypeImmediate::GreaterThanOrEqual => self.get_vector_compare_greater_than_or_equal_32(scan_constraint),
            ScanCompareTypeImmediate::LessThan => self.get_vector_compare_less_than_32(scan_constraint),
            ScanCompareTypeImmediate::LessThanOrEqual => self.get_vector_compare_less_than_or_equal_32(scan_constraint),
        }
    }

    fn get_vector_compare_func_immediate_16(
        &self,
        scan_compare_type: &ScanCompareTypeImmediate,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate16> {
        match scan_compare_type {
            ScanCompareTypeImmediate::Equal => self.get_vector_compare_equal_16(scan_constraint),
            ScanCompareTypeImmediate::NotEqual => self.get_vector_compare_not_equal_16(scan_constraint),
            ScanCompareTypeImmediate::GreaterThan => self.get_vector_compare_greater_than_16(scan_constraint),
            ScanCompareTypeImmediate::GreaterThanOrEqual => self.get_vector_compare_greater_than_or_equal_16(scan_constraint),
            ScanCompareTypeImmediate::LessThan => self.get_vector_compare_less_than_16(scan_constraint),
            ScanCompareTypeImmediate::LessThanOrEqual => self.get_vector_compare_less_than_or_equal_16(scan_constraint),
        }
    }

    fn get_vector_compare_func_relative_64(
        &self,
        scan_compare_type: &ScanCompareTypeRelative,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative64> {
        match scan_compare_type {
            ScanCompareTypeRelative::Changed => self.get_vector_compare_changed_64(scan_constraint),
            ScanCompareTypeRelative::Unchanged => self.get_vector_compare_unchanged_64(scan_constraint),
            ScanCompareTypeRelative::Increased => self.get_vector_compare_increased_64(scan_constraint),
            ScanCompareTypeRelative::Decreased => self.get_vector_compare_decreased_64(scan_constraint),
        }
    }

    fn get_vector_compare_func_relative_32(
        &self,
        scan_compare_type: &ScanCompareTypeRelative,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative32> {
        match scan_compare_type {
            ScanCompareTypeRelative::Changed => self.get_vector_compare_changed_32(scan_constraint),
            ScanCompareTypeRelative::Unchanged => self.get_vector_compare_unchanged_32(scan_constraint),
            ScanCompareTypeRelative::Increased => self.get_vector_compare_increased_32(scan_constraint),
            ScanCompareTypeRelative::Decreased => self.get_vector_compare_decreased_32(scan_constraint),
        }
    }

    fn get_vector_compare_func_relative_16(
        &self,
        scan_compare_type: &ScanCompareTypeRelative,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative16> {
        match scan_compare_type {
            ScanCompareTypeRelative::Changed => self.get_vector_compare_changed_16(scan_constraint),
            ScanCompareTypeRelative::Unchanged => self.get_vector_compare_unchanged_16(scan_constraint),
            ScanCompareTypeRelative::Increased => self.get_vector_compare_increased_16(scan_constraint),
            ScanCompareTypeRelative::Decreased => self.get_vector_compare_decreased_16(scan_constraint),
        }
    }

    fn get_vector_compare_func_delta_64(
        &self,
        scan_compare_type: &ScanCompareTypeDelta,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta64> {
        match scan_compare_type {
            ScanCompareTypeDelta::IncreasedByX => self.get_vector_compare_increased_by_64(scan_constraint),
            ScanCompareTypeDelta::DecreasedByX => self.get_vector_compare_decreased_by_64(scan_constraint),
            ScanCompareTypeDelta::MultipliedByX => self.get_vector_compare_multiplied_by_64(scan_constraint),
            ScanCompareTypeDelta::DividedByX => self.get_vector_compare_divided_by_64(scan_constraint),
            ScanCompareTypeDelta::ModuloByX => self.get_vector_compare_modulo_by_64(scan_constraint),
            ScanCompareTypeDelta::ShiftLeftByX => self.get_vector_compare_shift_left_by_64(scan_constraint),
            ScanCompareTypeDelta::ShiftRightByX => self.get_vector_compare_shift_right_by_64(scan_constraint),
            ScanCompareTypeDelta::LogicalAndByX => self.get_vector_compare_logical_and_by_64(scan_constraint),
            ScanCompareTypeDelta::LogicalOrByX => self.get_vector_compare_logical_or_by_64(scan_constraint),
            ScanCompareTypeDelta::LogicalXorByX => self.get_vector_compare_logical_xor_by_64(scan_constraint),
        }
    }

    fn get_vector_compare_func_delta_32(
        &self,
        scan_compare_type: &ScanCompareTypeDelta,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta32> {
        match scan_compare_type {
            ScanCompareTypeDelta::IncreasedByX => self.get_vector_compare_increased_by_32(scan_constraint),
            ScanCompareTypeDelta::DecreasedByX => self.get_vector_compare_decreased_by_32(scan_constraint),
            ScanCompareTypeDelta::MultipliedByX => self.get_vector_compare_multiplied_by_32(scan_constraint),
            ScanCompareTypeDelta::DividedByX => self.get_vector_compare_divided_by_32(scan_constraint),
            ScanCompareTypeDelta::ModuloByX => self.get_vector_compare_modulo_by_32(scan_constraint),
            ScanCompareTypeDelta::ShiftLeftByX => self.get_vector_compare_shift_left_by_32(scan_constraint),
            ScanCompareTypeDelta::ShiftRightByX => self.get_vector_compare_shift_right_by_32(scan_constraint),
            ScanCompareTypeDelta::LogicalAndByX => self.get_vector_compare_logical_and_by_32(scan_constraint),
            ScanCompareTypeDelta::LogicalOrByX => self.get_vector_compare_logical_or_by_32(scan_constraint),
            ScanCompareTypeDelta::LogicalXorByX => self.get_vector_compare_logical_xor_by_32(scan_constraint),
        }
    }

    fn get_vector_compare_func_delta_16(
        &self,
        scan_compare_type: &ScanCompareTypeDelta,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta16> {
        match scan_compare_type {
            ScanCompareTypeDelta::IncreasedByX => self.get_vector_compare_increased_by_16(scan_constraint),
            ScanCompareTypeDelta::DecreasedByX => self.get_vector_compare_decreased_by_16(scan_constraint),
            ScanCompareTypeDelta::MultipliedByX => self.get_vector_compare_multiplied_by_16(scan_constraint),
            ScanCompareTypeDelta::DividedByX => self.get_vector_compare_divided_by_16(scan_constraint),
            ScanCompareTypeDelta::ModuloByX => self.get_vector_compare_modulo_by_16(scan_constraint),
            ScanCompareTypeDelta::ShiftLeftByX => self.get_vector_compare_shift_left_by_16(scan_constraint),
            ScanCompareTypeDelta::ShiftRightByX => self.get_vector_compare_shift_right_by_16(scan_constraint),
            ScanCompareTypeDelta::LogicalAndByX => self.get_vector_compare_logical_and_by_16(scan_constraint),
            ScanCompareTypeDelta::LogicalOrByX => self.get_vector_compare_logical_or_by_16(scan_constraint),
            ScanCompareTypeDelta::LogicalXorByX => self.get_vector_compare_logical_xor_by_16(scan_constraint),
        }
    }
}
