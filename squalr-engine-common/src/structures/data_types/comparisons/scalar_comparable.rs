use crate::structures::scanning::scan_compare_type_delta::ScanCompareTypeDelta;
use crate::structures::scanning::scan_compare_type_immediate::ScanCompareTypeImmediate;
use crate::structures::scanning::scan_compare_type_relative::ScanCompareTypeRelative;

/// Defines a compare function that operates on an immediate (ie all inequalities).
pub type ScalarCompareFnImmediate = unsafe fn(
    // Current value pointer.
    *const u8,
    // Immediate value pointer.
    *const u8,
) -> bool;

/// Defines a compare function that operates on current and previous values (ie changed, unchanged, increased, decreased).
pub type ScalarCompareFnRelative = unsafe fn(
    // Current value pointer.
    *const u8,
    // Previous value pointer.
    *const u8,
) -> bool;

/// Defines a compare function that operates on current and previous values, with a delta arg (ie +x, -x).
pub type ScalarCompareFnDelta = unsafe fn(
    // Current value pointer.
    *const u8,
    // Previous value pointer.
    *const u8,
    // Delta value pointer.
    *const u8,
) -> bool;

pub trait ScalarComparable {
    fn get_compare_equal(&self) -> ScalarCompareFnRelative;
    fn get_compare_not_equal(&self) -> ScalarCompareFnRelative;
    fn get_compare_greater_than(&self) -> ScalarCompareFnRelative;
    fn get_compare_greater_than_or_equal(&self) -> ScalarCompareFnRelative;
    fn get_compare_less_than(&self) -> ScalarCompareFnRelative;
    fn get_compare_less_than_or_equal(&self) -> ScalarCompareFnRelative;

    fn get_compare_changed(&self) -> ScalarCompareFnRelative;
    fn get_compare_unchanged(&self) -> ScalarCompareFnRelative;
    fn get_compare_increased(&self) -> ScalarCompareFnRelative;
    fn get_compare_decreased(&self) -> ScalarCompareFnRelative;

    fn get_compare_increased_by(&self) -> ScalarCompareFnDelta;
    fn get_compare_decreased_by(&self) -> ScalarCompareFnDelta;

    fn get_scalar_compare_func_immediate(
        &self,
        scan_compare_type: ScanCompareTypeImmediate,
    ) -> ScalarCompareFnImmediate {
        match scan_compare_type {
            ScanCompareTypeImmediate::Equal => self.get_compare_equal(),
            ScanCompareTypeImmediate::NotEqual => self.get_compare_not_equal(),
            ScanCompareTypeImmediate::GreaterThan => self.get_compare_greater_than(),
            ScanCompareTypeImmediate::GreaterThanOrEqual => self.get_compare_greater_than_or_equal(),
            ScanCompareTypeImmediate::LessThan => self.get_compare_less_than(),
            ScanCompareTypeImmediate::LessThanOrEqual => self.get_compare_less_than_or_equal(),
        }
    }

    fn get_scalar_compare_function_relative(
        &self,
        scan_compare_type: ScanCompareTypeRelative,
    ) -> ScalarCompareFnRelative {
        match scan_compare_type {
            ScanCompareTypeRelative::Changed => self.get_compare_changed(),
            ScanCompareTypeRelative::Unchanged => self.get_compare_unchanged(),
            ScanCompareTypeRelative::Increased => self.get_compare_increased(),
            ScanCompareTypeRelative::Decreased => self.get_compare_decreased(),
        }
    }

    fn get_scalar_compare_function_delta(
        &self,
        scan_compare_type: ScanCompareTypeDelta,
    ) -> ScalarCompareFnDelta {
        match scan_compare_type {
            ScanCompareTypeDelta::IncreasedByX => self.get_compare_increased_by(),
            ScanCompareTypeDelta::DecreasedByX => self.get_compare_decreased_by(),
        }
    }
}
