use crate::structures::scanning::scan_parameters::ScanParameters;

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
    fn get_compare_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative;
    fn get_compare_not_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative;
    fn get_compare_greater_than(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative;
    fn get_compare_greater_than_or_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative;
    fn get_compare_less_than(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative;
    fn get_compare_less_than_or_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative;

    fn get_compare_changed(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative;
    fn get_compare_unchanged(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative;
    fn get_compare_increased(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative;
    fn get_compare_decreased(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnRelative;

    fn get_compare_increased_by(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnDelta;
    fn get_compare_decreased_by(
        &self,
        scan_parameters: &ScanParameters,
    ) -> ScalarCompareFnDelta;
}
