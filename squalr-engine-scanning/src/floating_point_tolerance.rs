#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatingPointTolerance {
    /// Represents a tolerance of 0, i.e., an exact floating-point match.
    ExactMatch,
    /// Represents a tolerance of 0.01.
    Tolerance10E2,
    /// Represents a tolerance of 0.001.
    Tolerance10E3,
    /// Represents a tolerance of 0.0001.
    Tolerance10E4,
    /// Represents a tolerance of 0.00001.
    Tolerance10E5,
}

impl Default for FloatingPointTolerance {
    fn default() -> Self {
        FloatingPointTolerance::Tolerance10E3
    }
}
