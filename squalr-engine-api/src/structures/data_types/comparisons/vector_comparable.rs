use crate::structures::scanning::comparisons::scan_compare_type_delta::ScanCompareTypeDelta;
use crate::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use crate::structures::scanning::comparisons::scan_compare_type_relative::ScanCompareTypeRelative;
use crate::structures::scanning::parameters::mapped_scan_parameters::ScanParametersCommon;
use std::simd::Simd;

/// Defines a compare function that operates on an immediate (ie all inequalities).
/// Parameters: current value pointer.
pub type VectorCompareFnImmediate64 = Box<dyn Fn(*const u8) -> Simd<u8, 64>>;
pub type VectorCompareFnImmediate32 = Box<dyn Fn(*const u8) -> Simd<u8, 32>>;
pub type VectorCompareFnImmediate16 = Box<dyn Fn(*const u8) -> Simd<u8, 16>>;

/// Defines a compare function that operates on current and previous values (ie changed, unchanged, increased, decreased).
/// Parameters: current value pointer, previous value pointer.
pub type VectorCompareFnRelative64 = Box<dyn Fn(*const u8, *const u8) -> Simd<u8, 64>>;
pub type VectorCompareFnRelative32 = Box<dyn Fn(*const u8, *const u8) -> Simd<u8, 32>>;
pub type VectorCompareFnRelative16 = Box<dyn Fn(*const u8, *const u8) -> Simd<u8, 16>>;

/// Defines a compare function that operates on current and previous values, with a delta arg (ie +x, -x).
/// Parameters: current value pointer, previous value pointer.
pub type VectorCompareFnDelta64 = Box<dyn Fn(*const u8, *const u8) -> Simd<u8, 64>>;
pub type VectorCompareFnDelta32 = Box<dyn Fn(*const u8, *const u8) -> Simd<u8, 32>>;
pub type VectorCompareFnDelta16 = Box<dyn Fn(*const u8, *const u8) -> Simd<u8, 16>>;

pub trait VectorComparable {
    fn get_vector_compare_equal_64(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnImmediate64>;
    fn get_vector_compare_equal_32(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnImmediate32>;
    fn get_vector_compare_equal_16(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnImmediate16>;
    fn get_vector_compare_not_equal_64(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnImmediate64>;
    fn get_vector_compare_not_equal_32(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnImmediate32>;
    fn get_vector_compare_not_equal_16(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnImmediate16>;
    fn get_vector_compare_greater_than_64(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnImmediate64>;
    fn get_vector_compare_greater_than_32(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnImmediate32>;
    fn get_vector_compare_greater_than_16(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnImmediate16>;
    fn get_vector_compare_greater_than_or_equal_64(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnImmediate64>;
    fn get_vector_compare_greater_than_or_equal_32(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnImmediate32>;
    fn get_vector_compare_greater_than_or_equal_16(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnImmediate16>;
    fn get_vector_compare_less_than_64(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnImmediate64>;
    fn get_vector_compare_less_than_32(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnImmediate32>;
    fn get_vector_compare_less_than_16(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnImmediate16>;
    fn get_vector_compare_less_than_or_equal_64(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnImmediate64>;
    fn get_vector_compare_less_than_or_equal_32(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnImmediate32>;
    fn get_vector_compare_less_than_or_equal_16(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnImmediate16>;

    fn get_vector_compare_changed_64(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnRelative64>;
    fn get_vector_compare_changed_32(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnRelative32>;
    fn get_vector_compare_changed_16(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnRelative16>;
    fn get_vector_compare_unchanged_64(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnRelative64>;
    fn get_vector_compare_unchanged_32(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnRelative32>;
    fn get_vector_compare_unchanged_16(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnRelative16>;
    fn get_vector_compare_increased_64(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnRelative64>;
    fn get_vector_compare_increased_32(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnRelative32>;
    fn get_vector_compare_increased_16(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnRelative16>;
    fn get_vector_compare_decreased_64(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnRelative64>;
    fn get_vector_compare_decreased_32(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnRelative32>;
    fn get_vector_compare_decreased_16(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnRelative16>;

    fn get_vector_compare_increased_by_64(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnDelta64>;
    fn get_vector_compare_increased_by_32(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnDelta32>;
    fn get_vector_compare_increased_by_16(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnDelta16>;
    fn get_vector_compare_decreased_by_64(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnDelta64>;
    fn get_vector_compare_decreased_by_32(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnDelta32>;
    fn get_vector_compare_decreased_by_16(
        &self,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnDelta16>;

    fn get_vector_compare_func_immediate_64(
        &self,
        scan_compare_type: &ScanCompareTypeImmediate,
        scan_parameters: &ScanParametersCommon,
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
        scan_parameters: &ScanParametersCommon,
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
        scan_parameters: &ScanParametersCommon,
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
        scan_parameters: &ScanParametersCommon,
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
        scan_parameters: &ScanParametersCommon,
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
        scan_parameters: &ScanParametersCommon,
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
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnDelta64> {
        match scan_compare_type {
            ScanCompareTypeDelta::IncreasedByX => self.get_vector_compare_increased_by_64(scan_parameters),
            ScanCompareTypeDelta::DecreasedByX => self.get_vector_compare_decreased_by_64(scan_parameters),
        }
    }

    fn get_vector_compare_func_delta_32(
        &self,
        scan_compare_type: &ScanCompareTypeDelta,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnDelta32> {
        match scan_compare_type {
            ScanCompareTypeDelta::IncreasedByX => self.get_vector_compare_increased_by_32(scan_parameters),
            ScanCompareTypeDelta::DecreasedByX => self.get_vector_compare_decreased_by_32(scan_parameters),
        }
    }

    fn get_vector_compare_func_delta_16(
        &self,
        scan_compare_type: &ScanCompareTypeDelta,
        scan_parameters: &ScanParametersCommon,
    ) -> Option<VectorCompareFnDelta16> {
        match scan_compare_type {
            ScanCompareTypeDelta::IncreasedByX => self.get_vector_compare_increased_by_16(scan_parameters),
            ScanCompareTypeDelta::DecreasedByX => self.get_vector_compare_decreased_by_16(scan_parameters),
        }
    }
}
