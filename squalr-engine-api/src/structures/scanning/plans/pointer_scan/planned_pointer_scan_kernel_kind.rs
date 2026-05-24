#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlannedPointerScanKernelKind {
    ScalarLinear,
    ScalarBinary,
    SimdLinear,
}

impl PlannedPointerScanKernelKind {
    pub fn get_display_name(self) -> &'static str {
        match self {
            Self::ScalarLinear => "Scalar Linear",
            Self::ScalarBinary => "Scalar Binary",
            Self::SimdLinear => "SIMD Linear",
        }
    }
}
