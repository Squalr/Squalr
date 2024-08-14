use crate::scanners::comparers::scalar::snapshot_element_scanner_scalar::SnapshotElementRangeScannerScalar;
use crate::scanners::comparers::snapshot_element_range_scanner::SnapshotElementRangeScanner;
use crate::scanners::comparers::snapshot_element_run_length_encoder::SnapshotElementRunLengthEncoder;
use crate::scanners::constraints::scan_constraint::ScanConstraint;
use crate::snapshots::snapshot_element_range::SnapshotElementRange;
use std::sync::{Arc, Once, RwLock};

pub struct SnapshotElementRangeScannerScalarIterative {
    scalar_scanner: SnapshotElementRangeScannerScalar,
}

impl SnapshotElementRangeScannerScalarIterative {
    fn new() -> Self {
        Self {
            scalar_scanner: SnapshotElementRangeScannerScalar::new(),
        }
    }
    
    pub fn get_instance() -> Arc<RwLock<SnapshotElementRangeScannerScalarIterative>> {
        static mut INSTANCE: Option<Arc<RwLock<SnapshotElementRangeScannerScalarIterative>>> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = Arc::new(RwLock::new(SnapshotElementRangeScannerScalarIterative::new()));
                INSTANCE = Some(instance);
            });

            return INSTANCE.as_ref().unwrap().clone();
        }
    }
}

impl SnapshotElementRangeScanner for SnapshotElementRangeScannerScalarIterative {
    fn scan_region(&mut self, element_range: &Arc<RwLock<SnapshotElementRange>>, constraint: Arc<ScanConstraint>) -> Vec<Arc<RwLock<SnapshotElementRange>>> {
        let element_range = element_range.read().unwrap();
        let mut current_value_pointer = element_range.get_current_values_pointer();
        let mut previous_value_pointer = element_range.get_previous_values_pointer();
        let mut run_length_encoder = SnapshotElementRunLengthEncoder::new();
        let aligned_element_count = element_range.get_aligned_element_count(constraint.get_byte_alignment());
        let data_type = constraint.get_element_type();

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
