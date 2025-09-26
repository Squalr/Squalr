use std::simd::{LaneCount, Simd, SupportedLaneCount};

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

/// Defines a vector compare function for immediate comparisons.
/// Takes a pointer to the current value and returns a SIMD result.
pub type VectorCompareFnImmediate<const N: usize> = Box<dyn Fn(*const u8) -> Simd<u8, N> + 'static>;

/// Defines a vector compare function for relative comparisons (e.g. changed, increased).
/// Takes a pointer to the current and previous values and returns a SIMD result.
pub type VectorCompareFnRelative<const N: usize> = Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N> + 'static>;

/// Enum that wraps vector comparison functions based on scan type.
pub enum ScanFunctionVector<const N: usize>
where
    LaneCount<N>: SupportedLaneCount,
{
    Immediate(VectorCompareFnImmediate<N>),
    RelativeOrDelta(VectorCompareFnRelative<N>),
}
