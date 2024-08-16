use std::str::FromStr;
use std::fmt::{self};

#[derive(Debug, Clone, PartialEq)]
pub enum ScanConstraintType {
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

impl FromStr for ScanConstraintType {
    type Err = ParseScanConstraintTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "==" => Ok(ScanConstraintType::Equal),
            "!=" => Ok(ScanConstraintType::NotEqual),
            "c" => Ok(ScanConstraintType::Changed),
            "u" => Ok(ScanConstraintType::Unchanged),
            "+" => Ok(ScanConstraintType::Increased),
            "-" => Ok(ScanConstraintType::Decreased),
            "+x" => Ok(ScanConstraintType::IncreasedByX),
            "-x" => Ok(ScanConstraintType::DecreasedByX),
            ">" => Ok(ScanConstraintType::GreaterThan),
            ">=" => Ok(ScanConstraintType::GreaterThanOrEqual),
            "<" => Ok(ScanConstraintType::LessThan),
            "<=" => Ok(ScanConstraintType::LessThanOrEqual),
            _ => Err(ParseScanConstraintTypeError),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseScanConstraintTypeError;

impl fmt::Display for ParseScanConstraintTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid constraint type")
    }
}

impl std::error::Error for ParseScanConstraintTypeError {}
