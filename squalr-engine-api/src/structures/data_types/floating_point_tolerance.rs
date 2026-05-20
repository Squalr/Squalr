use num_traits::Float;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Deserialize, PartialEq, Eq, Serialize)]
pub enum FloatingPointTolerance {
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
    /// Represents a tolerance of epsilon (ie essentially an exact match).
    #[serde(rename = "epsilon")]
    ToleranceEpsilon,
}

impl FloatingPointTolerance {
    pub fn get_value<PrimitiveType: Float>(&self) -> PrimitiveType {
        match self {
            FloatingPointTolerance::ToleranceEpsilon => PrimitiveType::epsilon(),
            FloatingPointTolerance::Tolerance10E1 => PrimitiveType::from(0.1).unwrap_or_else(PrimitiveType::epsilon),
            FloatingPointTolerance::Tolerance10E2 => PrimitiveType::from(0.01).unwrap_or_else(PrimitiveType::epsilon),
            FloatingPointTolerance::Tolerance10E3 => PrimitiveType::from(0.001).unwrap_or_else(PrimitiveType::epsilon),
            FloatingPointTolerance::Tolerance10E4 => PrimitiveType::from(0.0001).unwrap_or_else(PrimitiveType::epsilon),
            FloatingPointTolerance::Tolerance10E5 => PrimitiveType::from(0.00001).unwrap_or_else(PrimitiveType::epsilon),
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

impl FromStr for FloatingPointTolerance {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "0.1" => Ok(FloatingPointTolerance::Tolerance10E1),
            "0.01" => Ok(FloatingPointTolerance::Tolerance10E2),
            "0.001" => Ok(FloatingPointTolerance::Tolerance10E3),
            "0.0001" => Ok(FloatingPointTolerance::Tolerance10E4),
            "0.00001" => Ok(FloatingPointTolerance::Tolerance10E5),
            "epsilon" => Ok(FloatingPointTolerance::ToleranceEpsilon),
            _ => Err(format!("Invalid tolerance string: '{}'", string)),
        }
    }
}
