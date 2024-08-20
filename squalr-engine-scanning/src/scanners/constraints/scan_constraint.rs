use crate::scan_settings::ScanSettings;
use crate::scanners::constraints::scan_constraint_type::ScanConstraintType;
use squalr_engine_common::dynamic_struct::data_type::DataType;
use squalr_engine_common::dynamic_struct::data_value::DataValue;
use squalr_engine_memory::memory_alignment::MemoryAlignment;

#[derive(Debug, Clone)]
pub struct ScanConstraint {
    alignment: MemoryAlignment,
    constraint_type: ScanConstraintType,
    data_types: Vec<DataType>,
    constraint_value: Option<DataValue>,
}

impl ScanConstraint {
    pub fn new() -> Self {
        Self {
            alignment: MemoryAlignment::Auto,
            constraint_type: ScanConstraintType::Changed,
            data_types: vec![],
            constraint_value: None,
        }
    }

    pub fn new_with_value(
        alignment: MemoryAlignment,
        constraint_type: ScanConstraintType,
        data_types: Vec<DataType>,
        value: Option<DataValue>,
    ) -> Self {
        Self {
            alignment,
            constraint_type,
            data_types: data_types,
            constraint_value: value,
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

    pub fn get_constraint_value(&self) -> Option<&DataValue> {
        if self.is_immediate_constraint() {
            return self.constraint_value.as_ref();
        } else {
            return None;
        }
    }

    pub fn set_constraint_value(&mut self, value: Option<DataValue>) {
        self.constraint_value = value;
    }
    
    pub fn is_valid(&self) -> bool {
        if !self.is_immediate_constraint() {
            return true;
        } else {
            return self.constraint_value.is_some();
        }
    }

    pub fn is_relative_delta_constraint(&self) -> bool {
        return match self.constraint_type {
            | ScanConstraintType::IncreasedByX
            | ScanConstraintType::DecreasedByX => true,
            _ => false,
        };
    }

    pub fn is_relative_constraint(&self) -> bool {
        return match self.constraint_type {
            ScanConstraintType::Changed
            | ScanConstraintType::Unchanged
            | ScanConstraintType::Increased
            | ScanConstraintType::Decreased => true,
            _ => false,
        };
    }

    pub fn is_immediate_constraint(&self) -> bool {
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

    pub fn get_data_types(&self) -> &Vec<DataType> {
        return &self.data_types;
    }

    pub fn clone(&self) -> Self {
        ScanConstraint {
            alignment: self.alignment,
            constraint_type: self.constraint_type.clone(),
            data_types: self.data_types.clone(),
            constraint_value: self.constraint_value.clone(),
        }
    }

    pub fn conflicts_with(&self, other: &ScanConstraint) -> bool {
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
