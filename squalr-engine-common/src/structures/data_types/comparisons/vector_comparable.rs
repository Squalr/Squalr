use crate::structures::scanning::scan_compare_type_delta::ScanCompareTypeDelta;
use crate::structures::scanning::scan_compare_type_immediate::ScanCompareTypeImmediate;
use crate::structures::scanning::scan_compare_type_relative::ScanCompareTypeRelative;
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
    fn get_vector_compare_equal_64(&self) -> VectorCompareFnImmediate64;
    fn get_vector_compare_equal_32(&self) -> VectorCompareFnImmediate32;
    fn get_vector_compare_equal_16(&self) -> VectorCompareFnImmediate16;
    fn get_vector_compare_not_equal_64(&self) -> VectorCompareFnImmediate64;
    fn get_vector_compare_not_equal_32(&self) -> VectorCompareFnImmediate32;
    fn get_vector_compare_not_equal_16(&self) -> VectorCompareFnImmediate16;
    fn get_vector_compare_greater_than_64(&self) -> VectorCompareFnImmediate64;
    fn get_vector_compare_greater_than_32(&self) -> VectorCompareFnImmediate32;
    fn get_vector_compare_greater_than_16(&self) -> VectorCompareFnImmediate16;
    fn get_vector_compare_greater_than_or_equal_64(&self) -> VectorCompareFnImmediate64;
    fn get_vector_compare_greater_than_or_equal_32(&self) -> VectorCompareFnImmediate32;
    fn get_vector_compare_greater_than_or_equal_16(&self) -> VectorCompareFnImmediate16;
    fn get_vector_compare_less_than_64(&self) -> VectorCompareFnImmediate64;
    fn get_vector_compare_less_than_32(&self) -> VectorCompareFnImmediate32;
    fn get_vector_compare_less_than_16(&self) -> VectorCompareFnImmediate16;
    fn get_vector_compare_less_than_or_equal_64(&self) -> VectorCompareFnImmediate64;
    fn get_vector_compare_less_than_or_equal_32(&self) -> VectorCompareFnImmediate32;
    fn get_vector_compare_less_than_or_equal_16(&self) -> VectorCompareFnImmediate16;

    fn get_vector_compare_changed_64(&self) -> VectorCompareFnRelative64;
    fn get_vector_compare_changed_32(&self) -> VectorCompareFnRelative32;
    fn get_vector_compare_changed_16(&self) -> VectorCompareFnRelative16;
    fn get_vector_compare_unchanged_64(&self) -> VectorCompareFnRelative64;
    fn get_vector_compare_unchanged_32(&self) -> VectorCompareFnRelative32;
    fn get_vector_compare_unchanged_16(&self) -> VectorCompareFnRelative16;
    fn get_vector_compare_increased_64(&self) -> VectorCompareFnRelative64;
    fn get_vector_compare_increased_32(&self) -> VectorCompareFnRelative32;
    fn get_vector_compare_increased_16(&self) -> VectorCompareFnRelative16;
    fn get_vector_compare_decreased_64(&self) -> VectorCompareFnRelative64;
    fn get_vector_compare_decreased_32(&self) -> VectorCompareFnRelative32;
    fn get_vector_compare_decreased_16(&self) -> VectorCompareFnRelative16;

    fn get_vector_compare_increased_by_64(&self) -> VectorCompareFnDelta64;
    fn get_vector_compare_increased_by_32(&self) -> VectorCompareFnDelta32;
    fn get_vector_compare_increased_by_16(&self) -> VectorCompareFnDelta16;
    fn get_vector_compare_decreased_by_64(&self) -> VectorCompareFnDelta64;
    fn get_vector_compare_decreased_by_32(&self) -> VectorCompareFnDelta32;
    fn get_vector_compare_decreased_by_16(&self) -> VectorCompareFnDelta16;

    fn get_vector_compare_function_immediate_64(
        &self,
        scan_compare_type: ScanCompareTypeImmediate,
    ) -> VectorCompareFnImmediate64 {
        match scan_compare_type {
            ScanCompareTypeImmediate::Equal => self.get_vector_compare_equal_64(),
            ScanCompareTypeImmediate::NotEqual => self.get_vector_compare_not_equal_64(),
            ScanCompareTypeImmediate::GreaterThan => self.get_vector_compare_greater_than_64(),
            ScanCompareTypeImmediate::GreaterThanOrEqual => self.get_vector_compare_greater_than_or_equal_64(),
            ScanCompareTypeImmediate::LessThan => self.get_vector_compare_less_than_64(),
            ScanCompareTypeImmediate::LessThanOrEqual => self.get_vector_compare_less_than_or_equal_64(),
        }
    }

    fn get_vector_compare_function_immediate_32(
        &self,
        scan_compare_type: ScanCompareTypeImmediate,
    ) -> VectorCompareFnImmediate32 {
        match scan_compare_type {
            ScanCompareTypeImmediate::Equal => self.get_vector_compare_equal_32(),
            ScanCompareTypeImmediate::NotEqual => self.get_vector_compare_not_equal_32(),
            ScanCompareTypeImmediate::GreaterThan => self.get_vector_compare_greater_than_32(),
            ScanCompareTypeImmediate::GreaterThanOrEqual => self.get_vector_compare_greater_than_or_equal_32(),
            ScanCompareTypeImmediate::LessThan => self.get_vector_compare_less_than_32(),
            ScanCompareTypeImmediate::LessThanOrEqual => self.get_vector_compare_less_than_or_equal_32(),
        }
    }

    fn get_vector_compare_function_immediate_16(
        &self,
        scan_compare_type: ScanCompareTypeImmediate,
    ) -> VectorCompareFnImmediate16 {
        match scan_compare_type {
            ScanCompareTypeImmediate::Equal => self.get_vector_compare_equal_16(),
            ScanCompareTypeImmediate::NotEqual => self.get_vector_compare_not_equal_16(),
            ScanCompareTypeImmediate::GreaterThan => self.get_vector_compare_greater_than_16(),
            ScanCompareTypeImmediate::GreaterThanOrEqual => self.get_vector_compare_greater_than_or_equal_16(),
            ScanCompareTypeImmediate::LessThan => self.get_vector_compare_less_than_16(),
            ScanCompareTypeImmediate::LessThanOrEqual => self.get_vector_compare_less_than_or_equal_16(),
        }
    }

    fn get_vector_relative_compare_func_64(
        &self,
        scan_compare_type: ScanCompareTypeRelative,
    ) -> VectorCompareFnRelative64 {
        match scan_compare_type {
            ScanCompareTypeRelative::Changed => self.get_vector_compare_changed_64(),
            ScanCompareTypeRelative::Unchanged => self.get_vector_compare_unchanged_64(),
            ScanCompareTypeRelative::Increased => self.get_vector_compare_increased_64(),
            ScanCompareTypeRelative::Decreased => self.get_vector_compare_decreased_64(),
        }
    }

    fn get_vector_compare_function_delta_64(
        &self,
        scan_compare_type: ScanCompareTypeDelta,
    ) -> VectorCompareFnDelta64 {
        match scan_compare_type {
            ScanCompareTypeDelta::IncreasedByX => self.get_vector_compare_increased_by_64(),
            ScanCompareTypeDelta::DecreasedByX => self.get_vector_compare_decreased_by_64(),
        }
    }
}
