use std::{simd::Simd, sync::Arc};

/// Defines a compare function that operates on an immediate (ie all inequalities).
/// Parameters: current value pointer.
pub type VectorCompareFnImmediate64 = Arc<dyn Fn(*const u8) -> Simd<u8, 64> + Send + Sync + 'static>;
pub type VectorCompareFnImmediate32 = Arc<dyn Fn(*const u8) -> Simd<u8, 32> + Send + Sync + 'static>;
pub type VectorCompareFnImmediate16 = Arc<dyn Fn(*const u8) -> Simd<u8, 16> + Send + Sync + 'static>;

/// Defines a compare function that operates on current and previous values (ie changed, unchanged, increased, decreased).
/// Parameters: current value pointer, previous value pointer.
pub type VectorCompareFnRelative64 = Arc<dyn Fn(*const u8, *const u8) -> Simd<u8, 64> + Send + Sync + 'static>;
pub type VectorCompareFnRelative32 = Arc<dyn Fn(*const u8, *const u8) -> Simd<u8, 32> + Send + Sync + 'static>;
pub type VectorCompareFnRelative16 = Arc<dyn Fn(*const u8, *const u8) -> Simd<u8, 16> + Send + Sync + 'static>;

/// Defines a compare function that operates on current and previous values, with a delta arg (ie +x, -x).
/// Parameters: current value pointer, previous value pointer.
pub type VectorCompareFnDelta64 = Arc<dyn Fn(*const u8, *const u8) -> Simd<u8, 64> + Send + Sync + 'static>;
pub type VectorCompareFnDelta32 = Arc<dyn Fn(*const u8, *const u8) -> Simd<u8, 32> + Send + Sync + 'static>;
pub type VectorCompareFnDelta16 = Arc<dyn Fn(*const u8, *const u8) -> Simd<u8, 16> + Send + Sync + 'static>;

/// Defines a vector compare function for immediate comparisons.
/// Takes a pointer to the current value and returns a SIMD result.
pub type VectorCompareFnImmediate<const N: usize> = Arc<dyn Fn(*const u8) -> Simd<u8, N> + Send + Sync + 'static>;

/// Defines a vector compare function for relative comparisons (e.g. changed, increased).
/// Takes a pointer to the current and previous values and returns a SIMD result.
pub type VectorCompareFnRelative<const N: usize> = Arc<dyn Fn(*const u8, *const u8) -> Simd<u8, N> + Send + Sync + 'static>;

/// Defines a vector compare function that operates on current and previous values, with a delta arg (ie +x, -x).
/// Takes a pointer to the current and previous values and returns a SIMD result.
pub type VectorCompareFnDelta<const N: usize> = VectorCompareFnRelative<N>;

/// Enum that wraps vector comparison functions based on scan type.
pub enum ScanFunctionVector<const N: usize> {
    Immediate(VectorCompareFnImmediate<N>),
    RelativeOrDelta(VectorCompareFnRelative<N>),
}
