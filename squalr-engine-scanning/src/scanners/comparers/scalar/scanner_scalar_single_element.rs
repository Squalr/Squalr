use crate::snapshots::snapshot_region::SnapshotRegion;
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

/// Implements a scalar (ie CPU bound, non-SIMD) scanner which only scans a single element of memory (ie only containing 1 data type).
impl Scanner for ScannerScalarSingleElement {
    fn scan_region(&self, snapshot_region: &SnapshotRegion, snapshot_sub_region: &SnapshotSubRegion, constraint: &ScanConstraint) -> Vec<SnapshotSubRegion> {
        let current_value_pointer = snapshot_region.get_sub_region_current_values_pointer(&snapshot_sub_region);
        let previous_value_pointer = snapshot_region.get_sub_region_previous_values_pointer(&snapshot_sub_region);
        let data_type = constraint.get_element_type();

        if self.scalar_scanner.do_compare_action(current_value_pointer, previous_value_pointer, &constraint, &data_type) {
            let mut result = Vec::new();
            result.push(snapshot_sub_region.to_owned());
            return result;
        } else {
            return Vec::new();
        }
    }
}
