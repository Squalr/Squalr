use crate::scanners::constraints::scan_constraint_type::ScanConstraintType;
use squalr_engine_common::dynamic_struct::data_type::DataType;
use squalr_engine_common::dynamic_struct::data_value::DataValue;
use squalr_engine_memory::memory_alignment::MemoryAlignment;

#[derive(Debug, Clone)]
pub struct ScanFilterConstraint {
    alignment: Option<MemoryAlignment>,
    data_type: DataType,
}

impl Default for ScanFilterConstraint {
    fn default(
    ) -> Self {
        ScanFilterConstraint::new()
    }
}

impl ScanFilterConstraint {
    pub fn new() -> Self {
        Self {
            alignment: None,
            data_type: DataType::default(),
        }
    }

    pub fn new_with_value(
        alignment: Option<MemoryAlignment>,
        data_type: DataType,
    ) -> Self {
        Self {
            alignment: alignment,
            data_type: data_type,
        }
    }

    pub fn get_memory_alignment(
        &self
    ) -> &Option<MemoryAlignment>{
        return &self.alignment;
    }

    pub fn get_memory_alignment_or_default(
        &self,
        data_type: &DataType,
    ) -> MemoryAlignment{
        if let Some(alignment) = &self.alignment {
            return alignment.to_owned();
        }

        return MemoryAlignment::from(data_type.size_in_bytes() as i32);
    }

    pub fn get_data_type(
        &self
    ) -> &DataType{
        return &self.data_type;
    }
}

#[derive(Debug, Clone)]
pub struct ScanConstraint {
    constraint_type: ScanConstraintType,
    constraint_value: Option<DataValue>,
    scan_filter_constraints: Vec<ScanFilterConstraint>,
}

impl ScanConstraint {
    pub fn new() -> Self { // TODO: remove?
        Self {
            constraint_type: ScanConstraintType::Changed,
            constraint_value: None,
            scan_filter_constraints: vec![],
        }
    }

    pub fn new_with_value(
        constraint_type: ScanConstraintType,
        value: Option<DataValue>,
        scan_filter_constraints: Vec<ScanFilterConstraint>,
    ) -> Self {
        Self {
            constraint_type,
            constraint_value: value,
            scan_filter_constraints: scan_filter_constraints,
        }
    }
    
    pub fn get_constraint_type(
        &self
    ) -> ScanConstraintType {
        self.constraint_type.clone()
    }

    pub fn get_constraint_value(
        &self
    ) -> Option<&DataValue> {
        if self.is_immediate_constraint() {
            return self.constraint_value.as_ref();
        } else {
            return None;
        }
    }

    pub fn set_constraint_value(
        &mut self,
        value: Option<DataValue>
    ) {
        self.constraint_value = value;
    }
    
    pub fn is_valid(
        &self
    ) -> bool {
        if !self.is_immediate_constraint() {
            return true;
        } else {
            return self.constraint_value.is_some();
        }
    }

    pub fn is_relative_delta_constraint(
        &self
    ) -> bool {
        return match self.constraint_type {
            | ScanConstraintType::IncreasedByX
            | ScanConstraintType::DecreasedByX => true,
            _ => false,
        };
    }

    pub fn is_relative_constraint(
        &self
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
        &self
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

    pub fn get_scan_filter_constraints(
        &self
    ) -> &Vec<ScanFilterConstraint> {
        return &self.scan_filter_constraints;
    }

    pub fn clone(
        &self
    ) -> Self {
        ScanConstraint {
            constraint_type: self.constraint_type.clone(),
            constraint_value: self.constraint_value.clone(),
            scan_filter_constraints: self.scan_filter_constraints.clone(),
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
