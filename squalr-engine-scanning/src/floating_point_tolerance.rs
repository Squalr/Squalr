use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FloatingPointTolerance {
    /// Represents a tolerance of 0, i.e., an exact floating-point match.
    ExactMatch,
    /// Represents a tolerance of 0.1.
    Tolerance10E1,
    /// Represents a tolerance of 0.01.
    Tolerance10E2,
    /// Represents a tolerance of 0.001.
    Tolerance10E3,
    /// Represents a tolerance of 0.0001.
    Tolerance10E4,
    /// Represents a tolerance of 0.00001.
    Tolerance10E5,
}

impl fmt::Debug for FloatingPointTolerance {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let description = match self {
            FloatingPointTolerance::ExactMatch => "0.0",
            FloatingPointTolerance::Tolerance10E1 => "0.1",
            FloatingPointTolerance::Tolerance10E2 => "0.01",
            FloatingPointTolerance::Tolerance10E3 => "0.001",
            FloatingPointTolerance::Tolerance10E4 => "0.0001",
            FloatingPointTolerance::Tolerance10E5 => "0.00001",
        };
        write!(formatter, "{}", description)
    }
}

impl Default for FloatingPointTolerance {
    fn default() -> Self {
        FloatingPointTolerance::Tolerance10E3
    }
}
