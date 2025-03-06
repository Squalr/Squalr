use crate::structures::scanning::scan_compare_type_delta::ScanCompareTypeDelta;
use crate::structures::scanning::scan_compare_type_immediate::ScanCompareTypeImmediate;
use crate::structures::scanning::scan_compare_type_relative::ScanCompareTypeRelative;
use crate::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use std::simd::Simd;

/// Defines a compare function that operates on an immediate (ie all inequalities).
pub type VectorCompareFnImmediate64 = unsafe fn(
    // Current value pointer.
    *const u8,
    // Immediate value pointer.
    *const u8,
) -> Simd<u8, 64>;
pub type VectorCompareFnImmediate32 = unsafe fn(*const u8, *const u8) -> Simd<u8, 32>;
pub type VectorCompareFnImmediate16 = unsafe fn(*const u8, *const u8) -> Simd<u8, 16>;

/// Defines a compare function that operates on current and previous values (ie changed, unchanged, increased, decreased).
pub type VectorCompareFnRelative64 = unsafe fn(
    // Current value pointer.
    *const u8,
    // Previous value pointer.
    *const u8,
) -> Simd<u8, 64>;
pub type VectorCompareFnRelative32 = unsafe fn(*const u8, *const u8) -> Simd<u8, 32>;
pub type VectorCompareFnRelative16 = unsafe fn(*const u8, *const u8) -> Simd<u8, 16>;

/// Defines a compare function that operates on current and previous values, with a delta arg (ie +x, -x).
pub type VectorCompareFnDelta64 = unsafe fn(
    // Current value pointer.
    *const u8,
    // Previous value pointer.
    *const u8,
    // Delta value pointer.
    *const u8,
) -> Simd<u8, 64>;
pub type VectorCompareFnDelta32 = unsafe fn(*const u8, *const u8, *const u8) -> Simd<u8, 32>;
pub type VectorCompareFnDelta16 = unsafe fn(*const u8, *const u8, *const u8) -> Simd<u8, 16>;

pub trait VectorComparable {
    fn get_vector_compare_equal_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnImmediate64;
    fn get_vector_compare_equal_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnImmediate32;
    fn get_vector_compare_equal_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnImmediate16;
    fn get_vector_compare_not_equal_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnImmediate64;
    fn get_vector_compare_not_equal_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnImmediate32;
    fn get_vector_compare_not_equal_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnImmediate16;
    fn get_vector_compare_greater_than_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnImmediate64;
    fn get_vector_compare_greater_than_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnImmediate32;
    fn get_vector_compare_greater_than_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnImmediate16;
    fn get_vector_compare_greater_than_or_equal_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnImmediate64;
    fn get_vector_compare_greater_than_or_equal_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnImmediate32;
    fn get_vector_compare_greater_than_or_equal_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnImmediate16;
    fn get_vector_compare_less_than_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnImmediate64;
    fn get_vector_compare_less_than_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnImmediate32;
    fn get_vector_compare_less_than_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnImmediate16;
    fn get_vector_compare_less_than_or_equal_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnImmediate64;
    fn get_vector_compare_less_than_or_equal_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnImmediate32;
    fn get_vector_compare_less_than_or_equal_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnImmediate16;

    fn get_vector_compare_changed_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnRelative64;
    fn get_vector_compare_changed_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnRelative32;
    fn get_vector_compare_changed_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnRelative16;
    fn get_vector_compare_unchanged_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnRelative64;
    fn get_vector_compare_unchanged_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnRelative32;
    fn get_vector_compare_unchanged_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnRelative16;
    fn get_vector_compare_increased_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnRelative64;
    fn get_vector_compare_increased_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnRelative32;
    fn get_vector_compare_increased_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnRelative16;
    fn get_vector_compare_decreased_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnRelative64;
    fn get_vector_compare_decreased_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnRelative32;
    fn get_vector_compare_decreased_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnRelative16;

    fn get_vector_compare_increased_by_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnDelta64;
    fn get_vector_compare_increased_by_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnDelta32;
    fn get_vector_compare_increased_by_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnDelta16;
    fn get_vector_compare_decreased_by_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnDelta64;
    fn get_vector_compare_decreased_by_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnDelta32;
    fn get_vector_compare_decreased_by_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnDelta16;

    fn get_vector_compare_func_immediate_64(
        &self,
        scan_compare_type: &ScanCompareTypeImmediate,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnImmediate64 {
        match scan_compare_type {
            ScanCompareTypeImmediate::Equal => self.get_vector_compare_equal_64(scan_parameters_global),
            ScanCompareTypeImmediate::NotEqual => self.get_vector_compare_not_equal_64(scan_parameters_global),
            ScanCompareTypeImmediate::GreaterThan => self.get_vector_compare_greater_than_64(scan_parameters_global),
            ScanCompareTypeImmediate::GreaterThanOrEqual => self.get_vector_compare_greater_than_or_equal_64(scan_parameters_global),
            ScanCompareTypeImmediate::LessThan => self.get_vector_compare_less_than_64(scan_parameters_global),
            ScanCompareTypeImmediate::LessThanOrEqual => self.get_vector_compare_less_than_or_equal_64(scan_parameters_global),
        }
    }

    fn get_vector_compare_func_immediate_32(
        &self,
        scan_compare_type: &ScanCompareTypeImmediate,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnImmediate32 {
        match scan_compare_type {
            ScanCompareTypeImmediate::Equal => self.get_vector_compare_equal_32(scan_parameters_global),
            ScanCompareTypeImmediate::NotEqual => self.get_vector_compare_not_equal_32(scan_parameters_global),
            ScanCompareTypeImmediate::GreaterThan => self.get_vector_compare_greater_than_32(scan_parameters_global),
            ScanCompareTypeImmediate::GreaterThanOrEqual => self.get_vector_compare_greater_than_or_equal_32(scan_parameters_global),
            ScanCompareTypeImmediate::LessThan => self.get_vector_compare_less_than_32(scan_parameters_global),
            ScanCompareTypeImmediate::LessThanOrEqual => self.get_vector_compare_less_than_or_equal_32(scan_parameters_global),
        }
    }

    fn get_vector_compare_func_immediate_16(
        &self,
        scan_compare_type: &ScanCompareTypeImmediate,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnImmediate16 {
        match scan_compare_type {
            ScanCompareTypeImmediate::Equal => self.get_vector_compare_equal_16(scan_parameters_global),
            ScanCompareTypeImmediate::NotEqual => self.get_vector_compare_not_equal_16(scan_parameters_global),
            ScanCompareTypeImmediate::GreaterThan => self.get_vector_compare_greater_than_16(scan_parameters_global),
            ScanCompareTypeImmediate::GreaterThanOrEqual => self.get_vector_compare_greater_than_or_equal_16(scan_parameters_global),
            ScanCompareTypeImmediate::LessThan => self.get_vector_compare_less_than_16(scan_parameters_global),
            ScanCompareTypeImmediate::LessThanOrEqual => self.get_vector_compare_less_than_or_equal_16(scan_parameters_global),
        }
    }

    fn get_vector_compare_func_relative_64(
        &self,
        scan_compare_type: &ScanCompareTypeRelative,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnRelative64 {
        match scan_compare_type {
            ScanCompareTypeRelative::Changed => self.get_vector_compare_changed_64(scan_parameters_global),
            ScanCompareTypeRelative::Unchanged => self.get_vector_compare_unchanged_64(scan_parameters_global),
            ScanCompareTypeRelative::Increased => self.get_vector_compare_increased_64(scan_parameters_global),
            ScanCompareTypeRelative::Decreased => self.get_vector_compare_decreased_64(scan_parameters_global),
        }
    }

    fn get_vector_compare_func_relative_32(
        &self,
        scan_compare_type: &ScanCompareTypeRelative,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnRelative32 {
        match scan_compare_type {
            ScanCompareTypeRelative::Changed => self.get_vector_compare_changed_32(scan_parameters_global),
            ScanCompareTypeRelative::Unchanged => self.get_vector_compare_unchanged_32(scan_parameters_global),
            ScanCompareTypeRelative::Increased => self.get_vector_compare_increased_32(scan_parameters_global),
            ScanCompareTypeRelative::Decreased => self.get_vector_compare_decreased_32(scan_parameters_global),
        }
    }

    fn get_vector_compare_func_relative_16(
        &self,
        scan_compare_type: &ScanCompareTypeRelative,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnRelative16 {
        match scan_compare_type {
            ScanCompareTypeRelative::Changed => self.get_vector_compare_changed_16(scan_parameters_global),
            ScanCompareTypeRelative::Unchanged => self.get_vector_compare_unchanged_16(scan_parameters_global),
            ScanCompareTypeRelative::Increased => self.get_vector_compare_increased_16(scan_parameters_global),
            ScanCompareTypeRelative::Decreased => self.get_vector_compare_decreased_16(scan_parameters_global),
        }
    }

    fn get_vector_compare_func_delta_64(
        &self,
        scan_compare_type: &ScanCompareTypeDelta,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnDelta64 {
        match scan_compare_type {
            ScanCompareTypeDelta::IncreasedByX => self.get_vector_compare_increased_by_64(scan_parameters_global),
            ScanCompareTypeDelta::DecreasedByX => self.get_vector_compare_decreased_by_64(scan_parameters_global),
        }
    }

    fn get_vector_compare_func_delta_32(
        &self,
        scan_compare_type: &ScanCompareTypeDelta,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnDelta32 {
        match scan_compare_type {
            ScanCompareTypeDelta::IncreasedByX => self.get_vector_compare_increased_by_32(scan_parameters_global),
            ScanCompareTypeDelta::DecreasedByX => self.get_vector_compare_decreased_by_32(scan_parameters_global),
        }
    }

    fn get_vector_compare_func_delta_16(
        &self,
        scan_compare_type: &ScanCompareTypeDelta,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> VectorCompareFnDelta16 {
        match scan_compare_type {
            ScanCompareTypeDelta::IncreasedByX => self.get_vector_compare_increased_by_16(scan_parameters_global),
            ScanCompareTypeDelta::DecreasedByX => self.get_vector_compare_decreased_by_16(scan_parameters_global),
        }
    }
}
