use crate::scanners::constraints::scan_constraint::ScanConstraint;
use squalr_engine_common::dynamic_struct::field_value::FieldValue;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct ScanConstraints {
    alignment: MemoryAlignment,
    element_type: FieldValue,
    root_constraint: Option<Rc<RefCell<ScanConstraint>>>,
}

impl ScanConstraints {
    pub fn new(element_type: FieldValue, root_constraint: Option<Rc<RefCell<ScanConstraint>>>, alignment: MemoryAlignment) -> Self {
        let mut constraints = ScanConstraints {
            alignment,
            element_type: element_type.clone(),
            root_constraint,
        };
        constraints.set_element_type(element_type);
        constraints
    }

    pub fn get_root_constraint(&self) -> &Option<Rc<RefCell<ScanConstraint>>> {
        return &self.root_constraint;
    }

    pub fn set_root_constraint(&mut self, root_constraint: Option<Rc<RefCell<ScanConstraint>>>) {
        self.root_constraint = root_constraint;
    }

    pub fn get_alignment(&self) -> MemoryAlignment {
        return self.alignment;
    }

    pub fn set_alignment(&mut self, alignment: MemoryAlignment) {
        self.alignment = alignment;
    }

    pub fn get_element_type(&self) -> &FieldValue {
        return &self.element_type;
    }

    pub fn set_element_type(&mut self, element_type: FieldValue) {
        self.element_type = element_type.clone();
        if let Some(root_constraint) = &self.root_constraint {
            root_constraint.borrow_mut().set_element_type(&element_type);
        }
    }

    pub fn is_valid(&self) -> bool {
        if let Some(root_constraint) = &self.root_constraint {
            return root_constraint.borrow().is_valid();
        }
        false
    }

    pub fn clone(&self) -> Self {
        ScanConstraints {
            alignment: self.alignment,
            element_type: self.element_type.clone(),
            root_constraint: self.root_constraint.clone(),
        }
    }
}
