use crate::scanners::comparers::scalar::snapshot_element_scanner_scalar::SnapshotElementRangeScannerScalar;
use crate::scanners::constraints::scan_constraints::ScanConstraints;
use crate::snapshots::snapshot_element_range::SnapshotElementRange;
use std::sync::Arc;

pub struct SnapshotElementRangeScannerScalarSingleElement {
    scalar_scanner: SnapshotElementRangeScannerScalar,
}

impl SnapshotElementRangeScannerScalarSingleElement {
    pub fn new() -> Self {
        Self {
            scalar_scanner: SnapshotElementRangeScannerScalar::new(),
        }
    }

    pub fn scan_region(&mut self, element_range: Arc<SnapshotElementRange>, constraints: Arc<ScanConstraints>) -> Vec<Arc<SnapshotElementRange>> {
        let current_value_pointer: *mut u8;
        let previous_value_pointer: *mut u8;
        let current_values = element_range.parent_region.write().unwrap().current_values.as_mut_ptr();
        let previous_values = element_range.parent_region.write().unwrap().previous_values.as_mut_ptr();

        unsafe {
            current_value_pointer = current_values.add(element_range.get_region_offset());
            previous_value_pointer = previous_values.add(element_range.get_region_offset());
        }

        let root_constraint = constraints.get_root_constraint().as_ref().unwrap();
        let scan_constraint = root_constraint.read().unwrap();
        let data_type = constraints.get_element_type();

        if self.scalar_scanner.do_compare_action(current_value_pointer, previous_value_pointer, &scan_constraint, data_type) {
            let mut result = Vec::new();
            result.push(element_range.clone());
            return result;
        } else {
            return Vec::new();
        }
    }
}
