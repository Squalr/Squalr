use squalr_engine_common::dynamic_struct::field_value::FieldValue;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum OperationType {
    Or,
    And,
    Xor,
}

#[derive(Debug, Clone)]
pub struct OperationConstraint {
    binary_operation: OperationType,
    left: Option<Box<dyn ScanConstraint>>,
    right: Option<Box<dyn ScanConstraint>>,
}

impl OperationConstraint {
    pub fn new(
        operation: OperationType,
        left: Option<Box<dyn ScanConstraint>>,
        right: Option<Box<dyn ScanConstraint>>,
    ) -> Self {
        OperationConstraint {
            binary_operation: operation,
            left,
            right,
        }
    }

    pub fn get_binary_operation(&self) -> &OperationType {
        return &self.binary_operation;
    }

    pub fn get_left(&self) -> Option<&Box<dyn ScanConstraint>> {
        return self.left.as_ref();
    }

    pub fn set_left(&mut self, left: Option<Box<dyn ScanConstraint>>) {
        self.left = left;
    }

    pub fn get_right(&self) -> Option<&Box<dyn ScanConstraint>> {
        return self.right.as_ref();
    }

    pub fn set_right(&mut self, right: Option<Box<dyn ScanConstraint>>) {
        self.right = right;
    }

    pub fn set_element_type(&self, element_type: &FieldValue) {
        if let Some(ref left) = self.left {
            left.set_element_type(element_type);
        }
        if let Some(ref right) = self.right {
            right.set_element_type(element_type);
        }
    }

    pub fn is_valid(&self) -> bool {
        self.left
            .as_ref()
            .map_or(false, |left| left.is_valid())
            && self.right
                .as_ref()
                .map_or(false, |right| right.is_valid())
    }
}

pub trait ScanConstraint: ScanConstraintClone + Debug {
    fn set_element_type(&self, element_type: &FieldValue);
    fn is_valid(&self) -> bool;
}

pub trait ScanConstraintClone {
    fn clone_box(&self) -> Box<dyn ScanConstraint>;
}

impl<T> ScanConstraintClone for T
where
    T: 'static + ScanConstraint + Clone,
{
    fn clone_box(&self) -> Box<dyn ScanConstraint> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn ScanConstraint> {
    fn clone(&self) -> Box<dyn ScanConstraint> {
        self.clone_box()
    }
}

impl ScanConstraint for OperationConstraint {
    fn set_element_type(&self, element_type: &FieldValue) {
        self.set_element_type(element_type);
    }

    fn is_valid(&self) -> bool {
        self.is_valid()
    }
}
