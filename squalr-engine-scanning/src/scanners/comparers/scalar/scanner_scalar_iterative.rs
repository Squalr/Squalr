use crate::scanners::comparers::scalar::scanner_scalar::ScannerScalar;
use crate::scanners::comparers::snapshot_scanner::Scanner;
use crate::scanners::comparers::snapshot_sub_region_run_length_encoder::SnapshotSubRegionRunLengthEncoder;
use crate::scanners::constraints::scan_constraint::ScanConstraint;
use crate::snapshots::snapshot_sub_region::SnapshotSubRegion;
use std::sync::{Arc, Once, RwLock};

pub struct ScannerScalarIterative {
    scalar_scanner: ScannerScalar,
}

impl ScannerScalarIterative {
    fn new() -> Self {
        Self {
            scalar_scanner: ScannerScalar::new(),
        }
    }
    
    pub fn get_instance() -> Arc<RwLock<ScannerScalarIterative>> {
        static mut INSTANCE: Option<Arc<RwLock<ScannerScalarIterative>>> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = Arc::new(RwLock::new(ScannerScalarIterative::new()));
                INSTANCE = Some(instance);
            });

            return INSTANCE.as_ref().unwrap().clone();
        }
    }
}

impl Scanner for ScannerScalarIterative {
    fn scan_region(&self, snapshot_sub_region: &Arc<RwLock<SnapshotSubRegion>>, constraint: Arc<ScanConstraint>) -> Vec<Arc<RwLock<SnapshotSubRegion>>> {
        let snapshot_sub_region = snapshot_sub_region.read().unwrap();
        let mut current_value_pointer = snapshot_sub_region.get_current_values_pointer();
        let mut previous_value_pointer = snapshot_sub_region.get_previous_values_pointer();
        let mut run_length_encoder = SnapshotSubRegionRunLengthEncoder::new();
        let data_type = constraint.get_element_type();
        let aligned_element_count = snapshot_sub_region.get_element_count(data_type.size_in_bytes(), constraint.get_byte_alignment());

        for _ in 0..aligned_element_count {
            if self.scalar_scanner.do_compare_action(current_value_pointer, previous_value_pointer, &constraint, &data_type) {
                run_length_encoder.encode_range(constraint.get_byte_alignment() as usize);
            } else {
                run_length_encoder.finalize_current_encode_unchecked(constraint.get_byte_alignment() as usize);
            }

            unsafe {
                current_value_pointer = current_value_pointer.add(constraint.get_byte_alignment() as usize);
                previous_value_pointer = previous_value_pointer.add(constraint.get_byte_alignment() as usize);
            }
        }

        run_length_encoder.finalize_current_encode_unchecked(0);

        return run_length_encoder.get_collected_regions().clone();
    }
}
