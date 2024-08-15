use crate::scanners::comparers::scalar::scanner_scalar::ScannerScalar;
use crate::scanners::comparers::snapshot_scanner::Scanner;
use crate::scanners::constraints::scan_constraint::ScanConstraint;
use crate::snapshots::snapshot_sub_region::SnapshotSubRegion;
use std::sync::{Arc, Once, RwLock};

pub struct ScannerScalarSingleElement {
    scalar_scanner: ScannerScalar,
}

impl ScannerScalarSingleElement {
    fn new() -> Self {
        Self {
            scalar_scanner: ScannerScalar::new(),
        }
    }
    
    pub fn get_instance() -> Arc<RwLock<ScannerScalarSingleElement>> {
        static mut INSTANCE: Option<Arc<RwLock<ScannerScalarSingleElement>>> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = Arc::new(RwLock::new(ScannerScalarSingleElement::new()));
                INSTANCE = Some(instance);
            });

            return INSTANCE.as_ref().unwrap().clone();
        }
    }
}

impl Scanner for ScannerScalarSingleElement {
    fn scan_region(&self, snapshot_sub_region: &Arc<RwLock<SnapshotSubRegion>>, constraint: Arc<ScanConstraint>) -> Vec<Arc<RwLock<SnapshotSubRegion>>> {
        let snapshot_sub_region_read = snapshot_sub_region.read().unwrap();
        let current_value_pointer = snapshot_sub_region_read.get_current_values_pointer();
        let previous_value_pointer = snapshot_sub_region_read.get_previous_values_pointer();
        let data_type = constraint.get_element_type();

        if self.scalar_scanner.do_compare_action(current_value_pointer, previous_value_pointer, &constraint, &data_type) {
            let mut result = Vec::new();
            result.push(snapshot_sub_region.clone());
            return result;
        } else {
            return Vec::new();
        }
    }
}
