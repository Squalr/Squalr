use crate::scanners::comparers::scalar::snapshot_element_scanner_scalar::SnapshotElementRangeScannerScalar;
use crate::scanners::comparers::snapshot_element_run_length_encoder::SnapshotElementRunLengthEncoder;
use crate::scanners::constraints::scan_constraints::ScanConstraints;
use crate::snapshots::snapshot_element_range::SnapshotElementRange;
use std::sync::Arc;

pub struct SnapshotElementRangeScannerScalarIterative {
    scalar_scanner: SnapshotElementRangeScannerScalar,
}

impl SnapshotElementRangeScannerScalarIterative {
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
            current_value_pointer = current_values.add(element_range.region_offset);
            previous_value_pointer = previous_values.add(element_range.region_offset);
        }

        let mut run_length_encoder = SnapshotElementRunLengthEncoder::new();
        let aligned_element_count = element_range.get_aligned_element_count(constraints.get_byte_alignment());
        let root_constraint = constraints.get_root_constraint().as_ref().unwrap();
        let scan_constraint = root_constraint.read().unwrap();
        let data_type = constraints.get_element_type();

        for _ in 0..aligned_element_count {
            if self.scalar_scanner.do_compare_action(current_value_pointer, previous_value_pointer, &scan_constraint, data_type) {
                run_length_encoder.encode_range(constraints.get_byte_alignment() as usize);
            } else {
                run_length_encoder.finalize_current_encode_unchecked(constraints.get_byte_alignment() as usize);
            }

            unsafe {
                current_value_pointer.add(constraints.get_byte_alignment() as usize);
                previous_value_pointer.add(constraints.get_byte_alignment() as usize);
            }
        }

        run_length_encoder.finalize_current_encode_unchecked(0);

        return run_length_encoder.get_collected_regions().clone();
    }
}
