use crate::structures::scanning::scan_parameters::ScanParameters;

/// Defines a compare function that operates on an immediate (ie all inequalities).
/// Parameters: current value pointer, immediate value pointer.
pub type ScalarCompareFnImmediate = Box<dyn Fn(*const u8, *const u8) -> bool + 'static>;

/// Defines a compare function that operates on current and previous values (ie changed, unchanged, increased, decreased).
/// Parameters: current value pointer, previous value pointer.
pub type ScalarCompareFnRelative = Box<dyn Fn(*const u8, *const u8) -> bool + 'static>;

/// Defines a compare function that operates on current and previous values, with a delta arg (ie +x, -x).
/// Parameters: current value pointer, previous value pointer, delta value pointer.
pub type ScalarCompareFnDelta = Box<dyn Fn(*const u8, *const u8, *const u8) -> bool + 'static>;

pub trait ScalarComparable {
    fn get_compare_equal(
        &self,
        scan_parameters: &ScanParameters,
    ) -> Box<(dyn Fn(*const u8, *const u8) -> bool + 'static)>;
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
