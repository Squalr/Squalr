use crate::scanners::constraints::scan_constraint_type::ScanConstraintType;
use squalr_engine_common::values::anonymous_value::AnonymousValue;
use squalr_engine_common::values::data_type::DataType;
use squalr_engine_common::values::data_value::DataValue;

#[derive(Debug, Clone)]
pub struct ScanConstraint {
    constraint_type: ScanConstraintType,
    constraint_value: Option<AnonymousValue>,
}

impl ScanConstraint {
    pub fn new() -> Self {
        Self {
            constraint_type: ScanConstraintType::Changed,
            constraint_value: None,
        }
    }

    pub fn new_with_value(
        constraint_type: ScanConstraintType,
        value: Option<AnonymousValue>,
    ) -> Self {
        Self {
            constraint_type,
            constraint_value: value,
        }
    }
    
    pub fn get_constraint_type(
        &self,
    ) -> ScanConstraintType {
        self.constraint_type.clone()
    }

    pub fn deanonymize_type(
        &self,
        data_type: &DataType,
    ) -> Option<DataValue> {
        if let Some(value) = &self.constraint_value {
            return match value.deanonymize_type(data_type) {
                Ok(result) => Some(result),
                Err(_) => None,
            };
        }

        return None;
    }

    pub fn get_constraint_value(
        &self,
    ) -> Option<&AnonymousValue> {
        if self.is_immediate_constraint() {
            return self.constraint_value.as_ref();
        } else {
            return None;
        }
    }
    
    pub fn is_valid(
        &self,
    ) -> bool {
        if !self.is_immediate_constraint() {
            return true;
        } else {
            return self.constraint_value.is_some();
        }
    }

    pub fn is_relative_delta_constraint(
        &self,
    ) -> bool {
        return match self.constraint_type {
            | ScanConstraintType::IncreasedByX
            | ScanConstraintType::DecreasedByX => true,
            _ => false,
        };
    }

    pub fn is_relative_constraint(
        &self,
    ) -> bool {
        return match self.constraint_type {
            ScanConstraintType::Changed
            | ScanConstraintType::Unchanged
            | ScanConstraintType::Increased
            | ScanConstraintType::Decreased => true,
            _ => false,
        };
    }

    pub fn is_immediate_constraint(
        &self,
    ) -> bool {
        return match self.constraint_type {
            ScanConstraintType::Equal
            | ScanConstraintType::NotEqual
            | ScanConstraintType::GreaterThan
            | ScanConstraintType::GreaterThanOrEqual
            | ScanConstraintType::LessThan
            | ScanConstraintType::LessThanOrEqual
            | ScanConstraintType::IncreasedByX
            | ScanConstraintType::DecreasedByX => true,
            _ => false,
        };
    }

    pub fn clone(
        &self,
    ) -> Self {
        ScanConstraint {
            constraint_type: self.constraint_type.clone(),
            constraint_value: self.constraint_value.clone(),
        }
    }

    pub fn conflicts_with(
        &self,
        other: &ScanConstraint
    ) -> bool {
        if self.constraint_type == other.constraint_type {
            return true;
        }

        if !self.is_immediate_constraint() && !other.is_immediate_constraint() {
            return true;
        }

        if self.is_immediate_constraint() && other.is_immediate_constraint() {
            if (matches!(self.constraint_type, ScanConstraintType::LessThan | ScanConstraintType::LessThanOrEqual | ScanConstraintType::NotEqual)
                && matches!(other.constraint_type, ScanConstraintType::GreaterThan | ScanConstraintType::GreaterThanOrEqual | ScanConstraintType::NotEqual))
                || (matches!(self.constraint_type, ScanConstraintType::GreaterThan | ScanConstraintType::GreaterThanOrEqual | ScanConstraintType::NotEqual)
                    && matches!(other.constraint_type, ScanConstraintType::LessThan | ScanConstraintType::LessThanOrEqual | ScanConstraintType::NotEqual))
            {
                return true;
            }
        }

        return false;
    }
}
