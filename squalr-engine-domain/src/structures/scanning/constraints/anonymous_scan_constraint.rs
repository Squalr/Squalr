use crate::registries::symbols::symbol_registry::SymbolRegistry;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use crate::structures::data_values::anonymous_value_string::AnonymousValueString;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use crate::structures::scanning::comparisons::scan_compare_type_delta::ScanCompareTypeDelta;
use crate::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use crate::structures::scanning::comparisons::scan_compare_type_relative::ScanCompareTypeRelative;
use crate::structures::scanning::constraints::scan_constraint::ScanConstraint;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Represents a scan constraint containing a compare type and an anonymous value.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnonymousScanConstraint {
    scan_compare_type: ScanCompareType,
    anonymous_value_string: Option<AnonymousValueString>,
}

impl AnonymousScanConstraint {
    pub fn new(
        scan_compare_type: ScanCompareType,
        anonymous_value_string: Option<AnonymousValueString>,
    ) -> Self {
        Self {
            scan_compare_type,
            anonymous_value_string,
        }
    }

    pub fn get_scan_compare_type(&self) -> ScanCompareType {
        self.scan_compare_type
    }

    pub fn get_anonymous_value_string(&self) -> &Option<AnonymousValueString> {
        &self.anonymous_value_string
    }

    pub fn deanonymize_constraint(
        &self,
        data_type_ref: &DataTypeRef,
        floating_point_tolerance: FloatingPointTolerance,
    ) -> Option<ScanConstraint> {
        let symbol_registry = SymbolRegistry::get_instance();

        if let Some(anonymous_value_string) = &self.anonymous_value_string {
            match symbol_registry.deanonymize_value_string(&data_type_ref, &anonymous_value_string) {
                Ok(data_value) => return Some(ScanConstraint::new(self.scan_compare_type, data_value, floating_point_tolerance)),
                Err(error) => log::error!("Unable to parse value in anonymous constraint: {}", error),
            }
        }

        None
    }
}

impl FromStr for AnonymousScanConstraint {
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

        for (prefix, scan_compare_type, needs_value) in prefixes {
            if string.starts_with(prefix) {
                let rest = &string[prefix.len()..].trim();

                if !needs_value && !rest.is_empty() {
                    continue; // Skip to next prefix instead of err, but actually in logic, we err only if no match at end.
                }

                let anonymous_value_string = if rest.is_empty() {
                    None
                } else {
                    match rest.parse::<AnonymousValueString>() {
                        Ok(anonymous_value_string) => Some(anonymous_value_string),
                        Err(error) => {
                            log::error!("Failed to parse scan constraint: {}", error);
                            continue;
                        }
                    }
                };

                if needs_value && anonymous_value_string.is_none() {
                    continue;
                }

                return Ok(AnonymousScanConstraint {
                    scan_compare_type,
                    anonymous_value_string,
                });
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
