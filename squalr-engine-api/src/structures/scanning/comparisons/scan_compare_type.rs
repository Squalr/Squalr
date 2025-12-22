use crate::structures::scanning::comparisons::scan_compare_type_delta::ScanCompareTypeDelta;
use crate::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use crate::structures::scanning::comparisons::scan_compare_type_relative::ScanCompareTypeRelative;
use serde::{Deserialize, Serialize};
use std::fmt::{self};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScanCompareType {
    Immediate(ScanCompareTypeImmediate),
    Relative(ScanCompareTypeRelative),
    Delta(ScanCompareTypeDelta),
}

impl FromStr for ScanCompareType {
    type Err = ParseScanCompareTypeError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "==" => Ok(ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal)),
            "!=" => Ok(ScanCompareType::Immediate(ScanCompareTypeImmediate::NotEqual)),
            ">" => Ok(ScanCompareType::Immediate(ScanCompareTypeImmediate::GreaterThan)),
            ">=" => Ok(ScanCompareType::Immediate(ScanCompareTypeImmediate::GreaterThanOrEqual)),
            "<" => Ok(ScanCompareType::Immediate(ScanCompareTypeImmediate::LessThan)),
            "<=" => Ok(ScanCompareType::Immediate(ScanCompareTypeImmediate::LessThanOrEqual)),
            "c" => Ok(ScanCompareType::Relative(ScanCompareTypeRelative::Changed)),
            "u" => Ok(ScanCompareType::Relative(ScanCompareTypeRelative::Unchanged)),
            "+" => Ok(ScanCompareType::Relative(ScanCompareTypeRelative::Increased)),
            "-" => Ok(ScanCompareType::Relative(ScanCompareTypeRelative::Decreased)),
            "+x" => Ok(ScanCompareType::Delta(ScanCompareTypeDelta::IncreasedByX)),
            "-x" => Ok(ScanCompareType::Delta(ScanCompareTypeDelta::DecreasedByX)),
            _ => Err(ParseScanCompareTypeError),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseScanCompareTypeError;

impl fmt::Display for ParseScanCompareTypeError {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(formatter, "Invalid comparison type")
    }
}

impl std::error::Error for ParseScanCompareTypeError {}
