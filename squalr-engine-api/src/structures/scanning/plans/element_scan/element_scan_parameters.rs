use crate::structures::scanning::constraints::scan_constraint::ScanConstraint;

/// Represents the scan arguments for an element-wise scan over a single data type.
#[derive(Debug, Clone)]
pub struct ElementScanParameters {
    scan_constraints: Vec<ScanConstraint>,
}

impl ElementScanParameters {
    pub fn new(scan_constraints: Vec<ScanConstraint>) -> Self {
        Self { scan_constraints }
    }

    pub fn get_scan_constraints(&self) -> &Vec<ScanConstraint> {
        &self.scan_constraints
    }

    pub fn get_scan_constraints_mut(&mut self) -> &mut Vec<ScanConstraint> {
        &mut self.scan_constraints
    }
}
