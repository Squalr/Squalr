use crate::scan_settings::ScanSettings;
use squalr_engine_common::dynamic_struct::field_value::Endian;
use squalr_engine_common::dynamic_struct::field_value::FieldValue;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintType {
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

#[derive(Debug, Clone)]
pub struct ScanConstraint {
    alignment: MemoryAlignment,
    constraint_type: ConstraintType,
    constraint_value: Option<FieldValue>,
    constraint_args: Option<FieldValue>,
}

impl ScanConstraint {
    pub fn new() -> Self {
        Self {
            alignment: MemoryAlignment::Auto,
            constraint_type: ConstraintType::Changed,
            constraint_value: None,
            constraint_args: None,
        }
    }

    pub fn new_with_value(
        alignment: MemoryAlignment,
        constraint_type: ConstraintType,
        value: Option<FieldValue>,
        args: Option<FieldValue>,
    ) -> Self {
        Self {
            alignment,
            constraint_type,
            constraint_value: value,
            constraint_args: args,
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

    pub fn get_constraint_type(&self) -> ConstraintType {
        self.constraint_type.clone()
    }

    pub fn set_constraint(&mut self, constraint_type: ConstraintType) {
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

    pub fn get_constraint_args(&self) -> Option<&FieldValue> {
        if self.is_valued_constraint() {
            return self.constraint_args.as_ref();
        } else {
            return None;
        }
    }

    pub fn set_constraint_args(&mut self, args: Option<FieldValue>) {
        self.constraint_args = args;
    }

    pub fn get_constraint_name(&self) -> &'static str {
        match self.constraint_type {
            ConstraintType::Equal => "Equal",
            ConstraintType::NotEqual => "Not Equal",
            ConstraintType::GreaterThan => "Greater Than",
            ConstraintType::GreaterThanOrEqual => "Greater Than Or Equal",
            ConstraintType::LessThan => "Less Than",
            ConstraintType::LessThanOrEqual => "Less Than Or Equal",
            ConstraintType::Changed => "Changed",
            ConstraintType::Unchanged => "Unchanged",
            ConstraintType::Increased => "Increased",
            ConstraintType::Decreased => "Decreased",
            ConstraintType::IncreasedByX => "Increased By X",
            ConstraintType::DecreasedByX => "Decreased By X",
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
            ConstraintType::Changed
            | ConstraintType::Unchanged
            | ConstraintType::Increased
            | ConstraintType::Decreased
            | ConstraintType::IncreasedByX
            | ConstraintType::DecreasedByX => true,
            _ => false,
        };
    }

    pub fn is_valued_constraint(&self) -> bool {
        return match self.constraint_type {
            ConstraintType::Equal
            | ConstraintType::NotEqual
            | ConstraintType::GreaterThan
            | ConstraintType::GreaterThanOrEqual
            | ConstraintType::LessThan
            | ConstraintType::LessThanOrEqual
            | ConstraintType::IncreasedByX
            | ConstraintType::DecreasedByX => true,
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
            constraint_args: self.constraint_args.clone(),
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
                if (matches!(self.constraint_type, ConstraintType::LessThan | ConstraintType::LessThanOrEqual | ConstraintType::NotEqual)
                    && matches!(other.constraint_type, ConstraintType::GreaterThan | ConstraintType::GreaterThanOrEqual | ConstraintType::NotEqual))
                    || (matches!(self.constraint_type, ConstraintType::GreaterThan | ConstraintType::GreaterThanOrEqual | ConstraintType::NotEqual)
                        && matches!(other.constraint_type, ConstraintType::LessThan | ConstraintType::LessThanOrEqual | ConstraintType::NotEqual))
                {
                    return true;
                }
            }
        }

        return false;
    }
}

impl Display for ConstraintType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ConstraintType::Equal => "Equal",
                ConstraintType::NotEqual => "Not Equal",
                ConstraintType::Changed => "Changed",
                ConstraintType::Unchanged => "Unchanged",
                ConstraintType::Increased => "Increased",
                ConstraintType::Decreased => "Decreased",
                ConstraintType::IncreasedByX => "Increased By X",
                ConstraintType::DecreasedByX => "Decreased By X",
                ConstraintType::GreaterThan => "Greater Than",
                ConstraintType::GreaterThanOrEqual => "Greater Than Or Equal",
                ConstraintType::LessThan => "Less Than",
                ConstraintType::LessThanOrEqual => "Less Than Or Equal",
            }
        )
    }
}
