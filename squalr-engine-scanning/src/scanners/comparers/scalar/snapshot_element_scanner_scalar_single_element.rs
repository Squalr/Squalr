use crate::scanners::comparers::scalar::snapshot_element_scanner_scalar::SnapshotElementRangeScannerScalar;
use crate::scanners::comparers::snapshot_element_range_scanner::SnapshotElementRangeScanner;
use crate::scanners::constraints::scan_constraints::ScanConstraints;
use crate::snapshots::snapshot_element_range::SnapshotElementRange;
use std::sync::{Arc, Once, RwLock};

pub struct SnapshotElementRangeScannerScalarSingleElement {
    scalar_scanner: SnapshotElementRangeScannerScalar,
}

impl SnapshotElementRangeScannerScalarSingleElement {
    fn new() -> Self {
        Self {
            scalar_scanner: SnapshotElementRangeScannerScalar::new(),
        }
    }
    
    pub fn get_instance() -> Arc<RwLock<SnapshotElementRangeScannerScalarSingleElement>> {
        static mut INSTANCE: Option<Arc<RwLock<SnapshotElementRangeScannerScalarSingleElement>>> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = Arc::new(RwLock::new(SnapshotElementRangeScannerScalarSingleElement::new()));
                INSTANCE = Some(instance);
            });

            return INSTANCE.as_ref().unwrap().clone();
        }
    }
}

impl SnapshotElementRangeScanner for SnapshotElementRangeScannerScalarSingleElement {
    fn scan_region(&mut self, element_range: &Arc<RwLock<SnapshotElementRange>>, constraints: Arc<ScanConstraints>) -> Vec<Arc<RwLock<SnapshotElementRange>>> {
        let element_range_read = element_range.read().unwrap();
        let current_value_pointer = element_range_read.get_current_values_pointer();
        let previous_value_pointer = element_range_read.get_previous_values_pointer();
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
