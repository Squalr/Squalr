const SIMD_LINEAR_RANGE_THRESHOLD: usize = 8;
const SCALAR_LINEAR_RANGE_THRESHOLD: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PointerScanRangeSearchKernelKind {
    ScalarLinear,
    ScalarBinary,
    SimdLinear,
}

impl PointerScanRangeSearchKernelKind {
    pub(crate) fn from_target_range_count(target_range_count: usize) -> Self {
        if target_range_count <= SIMD_LINEAR_RANGE_THRESHOLD {
            Self::SimdLinear
        } else if target_range_count <= SCALAR_LINEAR_RANGE_THRESHOLD {
            Self::ScalarLinear
        } else {
            Self::ScalarBinary
        }
    }

    pub(crate) fn get_name(self) -> &'static str {
        match self {
            Self::ScalarLinear => "Scalar Linear",
            Self::ScalarBinary => "Scalar Binary",
            Self::SimdLinear => "SIMD Linear",
        }
    }
}
