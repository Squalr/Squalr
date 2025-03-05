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
}
