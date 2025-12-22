use crate::structures::data_values::anonymous_value::AnonymousValue;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use crate::structures::scanning::comparisons::scan_compare_type_delta::ScanCompareTypeDelta;
use crate::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use crate::structures::scanning::comparisons::scan_compare_type_relative::ScanCompareTypeRelative;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanConstraint {
    pub compare_type: ScanCompareType,
    pub value: Option<AnonymousValue>,
}

impl ScanConstraint {
    pub fn new(
        compare_type: ScanCompareType,
        value: Option<AnonymousValue>,
    ) -> Self {
        Self { compare_type, value }
    }
}

impl FromStr for ScanConstraint {
    type Err = ParseScanConstraintError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        // Prefixes are deliberately ordered in a manner to resolve ambiguity (ie + vs +{value}).
        let prefixes = [
            // Relative scans.
            ("!=", ScanCompareType::Relative(ScanCompareTypeRelative::Changed), false),
            ("==", ScanCompareType::Relative(ScanCompareTypeRelative::Unchanged), false),
            ("+", ScanCompareType::Relative(ScanCompareTypeRelative::Increased), false),
            ("-", ScanCompareType::Relative(ScanCompareTypeRelative::Decreased), false),
            // Immediate scans.
            (">=", ScanCompareType::Immediate(ScanCompareTypeImmediate::GreaterThanOrEqual), true),
            ("<=", ScanCompareType::Immediate(ScanCompareTypeImmediate::LessThanOrEqual), true),
            (">", ScanCompareType::Immediate(ScanCompareTypeImmediate::GreaterThan), true),
            ("<", ScanCompareType::Immediate(ScanCompareTypeImmediate::LessThan), true),
            ("==", ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal), true),
            ("!=", ScanCompareType::Immediate(ScanCompareTypeImmediate::NotEqual), true),
            // Delta scans.
            ("+", ScanCompareType::Delta(ScanCompareTypeDelta::IncreasedByX), true),
            ("-", ScanCompareType::Delta(ScanCompareTypeDelta::DecreasedByX), true),
        ];

        let string = string.trim();

        for (prefix, compare_type, needs_value) in prefixes {
            if string.starts_with(prefix) {
                let rest = &string[prefix.len()..].trim();

                if !needs_value && !rest.is_empty() {
                    continue; // Skip to next prefix instead of err, but actually in logic, we err only if no match at end.
                }

                let value = if rest.is_empty() {
                    None
                } else {
                    match rest.parse::<AnonymousValue>() {
                        Ok(anonymous_value) => Some(anonymous_value),
                        Err(error) => {
                            log::error!("Failed to parse scan constraint: {}", error);
                            continue;
                        }
                    }
                };

                if needs_value && value.is_none() {
                    continue;
                }

                return Ok(ScanConstraint { compare_type, value });
            }
        }

        Err(ParseScanConstraintError)
    }
}

#[derive(Debug, Clone)]
pub struct ParseScanConstraintError;

impl fmt::Display for ParseScanConstraintError {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(formatter, "Invalid scan constraint")
    }
}

impl std::error::Error for ParseScanConstraintError {}
