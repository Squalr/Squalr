use std::sync::Arc;

/// Defines a compare function that operates on an immediate (ie all inequalities).
/// Parameters: current value pointer.
pub type ScalarCompareFnImmediate = Arc<dyn Fn(*const u8) -> bool + Send + Sync + 'static>;

/// Defines a compare function that operates on current and previous values (ie changed, unchanged, increased, decreased).
/// Parameters: current value pointer, previous value pointer.
pub type ScalarCompareFnRelative = Arc<dyn Fn(*const u8, *const u8) -> bool + Send + Sync + 'static>;

/// Defines a compare function that operates on current and previous values, with a delta arg (ie +x, -x).
/// Parameters: current value pointer, previous value pointer.
pub type ScalarCompareFnDelta = ScalarCompareFnRelative;

pub enum ScanFunctionScalar {
    Immediate(ScalarCompareFnImmediate),
    RelativeOrDelta(ScalarCompareFnRelative),
}
