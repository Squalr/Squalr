use serde::{Deserialize, Serialize};
use std::fmt::{self};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScanCompareType {
    Equal,
    NotEqual,
    Changed,
    Unchanged,
    Increased,
    Decreased,
    IncreasedByX,
    DecreasedByX,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

impl FromStr for ScanCompareType {
    type Err = ParseScanCompareTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "==" => Ok(ScanCompareType::Equal),
            "!=" => Ok(ScanCompareType::NotEqual),
            "c" => Ok(ScanCompareType::Changed),
            "u" => Ok(ScanCompareType::Unchanged),
            "+" => Ok(ScanCompareType::Increased),
            "-" => Ok(ScanCompareType::Decreased),
            "+x" => Ok(ScanCompareType::IncreasedByX),
            "-x" => Ok(ScanCompareType::DecreasedByX),
            ">" => Ok(ScanCompareType::GreaterThan),
            ">=" => Ok(ScanCompareType::GreaterThanOrEqual),
            "<" => Ok(ScanCompareType::LessThan),
            "<=" => Ok(ScanCompareType::LessThanOrEqual),
            _ => Err(ParseScanCompareTypeError),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseScanCompareTypeError;

impl fmt::Display for ParseScanCompareTypeError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(f, "Invalid comparison type")
    }
}

impl std::error::Error for ParseScanCompareTypeError {}
