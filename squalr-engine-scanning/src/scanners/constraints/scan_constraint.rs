use crate::scan_settings::ScanSettings;
use crate::scanners::constraints::scan_constraint_type::ScanConstraintType;
use squalr_engine_common::dynamic_struct::field_value::Endian;
use squalr_engine_common::dynamic_struct::field_value::FieldValue;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::fmt::{self, Display};

#[derive(Debug, Clone)]
pub struct ScanConstraint {
    alignment: MemoryAlignment,
    constraint_type: ScanConstraintType,
    constraint_value: Option<FieldValue>,
    constraint_delta_value: Option<FieldValue>,
}

impl ScanConstraint {
    pub fn new() -> Self {
        Self {
            alignment: MemoryAlignment::Auto,
            constraint_type: ScanConstraintType::Changed,
            constraint_value: None,
            constraint_delta_value: None,
        }
    }

    pub fn new_with_value(
        alignment: MemoryAlignment,
        constraint_type: ScanConstraintType,
        value: Option<FieldValue>,
        delta_value: Option<FieldValue>,
    ) -> Self {
        Self {
            alignment,
            constraint_type,
            constraint_value: value,
            constraint_delta_value: delta_value,
        }
    }

    pub fn clone_and_resolve_auto_alignment(&self) -> ScanConstraint {
        let mut constraint = self.clone();

        if constraint.get_alignment() == MemoryAlignment::Auto {
            let settings_alignment = ScanSettings::get_instance().get_alignment();
            if settings_alignment == MemoryAlignment::Auto {
                constraint.set_alignment(MemoryAlignment::Alignment4);
            }
            else {
                constraint.set_alignment(settings_alignment);
            }
        }

        return constraint;
    }

    pub fn get_alignment(&self) -> MemoryAlignment {
        return self.alignment;
    }

    pub fn set_alignment(&mut self, alignment: MemoryAlignment) {
        self.alignment = alignment;
    }

    pub fn get_constraint_type(&self) -> ScanConstraintType {
        self.constraint_type.clone()
    }

    pub fn set_constraint(&mut self, constraint_type: ScanConstraintType) {
        self.constraint_type = constraint_type;
        self.constraint_value = self.constraint_value.clone(); // Force update of constraint value to determine if valid
    }

    pub fn get_constraint_value(&self) -> Option<&FieldValue> {
        if self.is_valued_constraint() {
            return self.constraint_value.as_ref();
        } else {
            return None;
        }
    }

    pub fn set_constraint_value(&mut self, value: Option<FieldValue>) {
        self.constraint_value = value;
    }

    pub fn get_constraint_delta_value(&self) -> Option<&FieldValue> {
        if self.is_valued_constraint() {
            return self.constraint_delta_value.as_ref();
        } else {
            return None;
        }
    }

    pub fn set_constraint_delta_value(&mut self, args: Option<FieldValue>) {
        self.constraint_delta_value = args;
    }

    pub fn get_constraint_name(&self) -> &'static str {
        match self.constraint_type {
            ScanConstraintType::Equal => "Equal",
            ScanConstraintType::NotEqual => "Not Equal",
            ScanConstraintType::GreaterThan => "Greater Than",
            ScanConstraintType::GreaterThanOrEqual => "Greater Than Or Equal",
            ScanConstraintType::LessThan => "Less Than",
            ScanConstraintType::LessThanOrEqual => "Less Than Or Equal",
            ScanConstraintType::Changed => "Changed",
            ScanConstraintType::Unchanged => "Unchanged",
            ScanConstraintType::Increased => "Increased",
            ScanConstraintType::Decreased => "Decreased",
            ScanConstraintType::IncreasedByX => "Increased By X",
            ScanConstraintType::DecreasedByX => "Decreased By X",
        }
    }

    pub fn is_valid(&self) -> bool {
        if !self.is_valued_constraint() {
            return true;
        } else {
            return self.constraint_value.is_some();
        }
    }

    pub fn is_relative_constraint(&self) -> bool {
        return match self.constraint_type {
            ScanConstraintType::Changed
            | ScanConstraintType::Unchanged
            | ScanConstraintType::Increased
            | ScanConstraintType::Decreased
            | ScanConstraintType::IncreasedByX
            | ScanConstraintType::DecreasedByX => true,
            _ => false,
        };
    }

    pub fn is_valued_constraint(&self) -> bool {
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

    pub fn set_element_type(&mut self, element_type: &FieldValue) {
        if self.constraint_value.is_none() {
            return;
        }

        let target_type = match element_type {
            FieldValue::U16(_, Endian::Big)
            | FieldValue::I16(_, Endian::Big)
            | FieldValue::U32(_, Endian::Big)
            | FieldValue::I32(_, Endian::Big)
            | FieldValue::U64(_, Endian::Big)
            | FieldValue::I64(_, Endian::Big)
            | FieldValue::F32(_, Endian::Big)
            | FieldValue::F64(_, Endian::Big) => Some(element_type),
            _ => None,
        };

        if let Some(target) = target_type {
            self.constraint_value = Some(target.clone());
        } else {
            self.constraint_value = None;
        }
    }

    pub fn get_element_type(&self) -> FieldValue {
        if let Some(constraint_value) = &self.constraint_value {
            return constraint_value.clone();
        } else {
            return FieldValue::U32(0, Endian::Big);
        }
    }

    pub fn clone(&self) -> Self {
        ScanConstraint {
            alignment: self.alignment,
            constraint_type: self.constraint_type.clone(),
            constraint_value: self.constraint_value.clone(),
            constraint_delta_value: self.constraint_delta_value.clone(),
        }
    }

    pub fn conflicts_with(&self, other: &ScanConstraint) -> bool {
        if self.constraint_type == other.constraint_type {
            return true;
        }

        if self.is_relative_constraint() && other.is_relative_constraint() {
            return true;
        }

        if self.is_valued_constraint() && other.is_valued_constraint() {
            if !self.is_relative_constraint() && !other.is_relative_constraint() {
                if (matches!(self.constraint_type, ScanConstraintType::LessThan | ScanConstraintType::LessThanOrEqual | ScanConstraintType::NotEqual)
                    && matches!(other.constraint_type, ScanConstraintType::GreaterThan | ScanConstraintType::GreaterThanOrEqual | ScanConstraintType::NotEqual))
                    || (matches!(self.constraint_type, ScanConstraintType::GreaterThan | ScanConstraintType::GreaterThanOrEqual | ScanConstraintType::NotEqual)
                        && matches!(other.constraint_type, ScanConstraintType::LessThan | ScanConstraintType::LessThanOrEqual | ScanConstraintType::NotEqual))
                {
                    return true;
                }
            }
        }

        return false;
    }
}
