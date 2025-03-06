use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::fmt;

#[derive(Clone, Copy, Deserialize, PartialEq, Eq, Serialize)]
pub enum FloatingPointTolerance {
    /// Represents a tolerance of epsilon (ie essentially an exact match).
    #[serde(rename = "exact")]
    ExactMatch,
    /// Represents a tolerance of 0.1.
    #[serde(rename = "0.1")]
    Tolerance10E1,
    /// Represents a tolerance of 0.01.
    #[serde(rename = "0.01")]
    Tolerance10E2,
    /// Represents a tolerance of 0.001.
    #[serde(rename = "0.001")]
    Tolerance10E3,
    /// Represents a tolerance of 0.0001.
    #[serde(rename = "0.0001")]
    Tolerance10E4,
    /// Represents a tolerance of 0.00001.
    #[serde(rename = "0.00001")]
    Tolerance10E5,
}

impl FloatingPointTolerance {
    pub fn get_value_f32(&self) -> f32 {
        match self {
            FloatingPointTolerance::ExactMatch => f32::EPSILON,
            FloatingPointTolerance::Tolerance10E1 => 0.1,
            FloatingPointTolerance::Tolerance10E2 => 0.01,
            FloatingPointTolerance::Tolerance10E3 => 0.001,
            FloatingPointTolerance::Tolerance10E4 => 0.0001,
            FloatingPointTolerance::Tolerance10E5 => 0.00001,
        }
    }

    pub fn get_value_f64(&self) -> f64 {
        match self {
            FloatingPointTolerance::ExactMatch => f64::EPSILON,
            FloatingPointTolerance::Tolerance10E1 => 0.1,
            FloatingPointTolerance::Tolerance10E2 => 0.01,
            FloatingPointTolerance::Tolerance10E3 => 0.001,
            FloatingPointTolerance::Tolerance10E4 => 0.0001,
            FloatingPointTolerance::Tolerance10E5 => 0.00001,
        }
    }
}

impl Default for FloatingPointTolerance {
    fn default() -> Self {
        FloatingPointTolerance::Tolerance10E3
    }
}

impl fmt::Debug for FloatingPointTolerance {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match to_string_pretty(&self) {
            Ok(json) => write!(formatter, "{}", json),
            Err(_) => write!(formatter, "FloatingPointTolerance {{ could not serialize to JSON }}"),
        }
    }
}
