use crate::scanners::comparers::scalar::scanner_scalar::ScannerScalar;
use crate::scanners::comparers::snapshot_scanner::Scanner;
use crate::scanners::comparers::snapshot_sub_region_run_length_encoder::SnapshotSubRegionRunLengthEncoder;
use crate::scanners::constraints::scan_constraint::ScanConstraint;
use crate::snapshots::snapshot_region::SnapshotRegion;
use crate::snapshots::snapshot_sub_region::SnapshotSubRegion;
use std::borrow::BorrowMut;
use std::sync::Once;

pub struct ScannerScalarIterative {
    scalar_scanner: ScannerScalar,
}

impl ScannerScalarIterative {
    fn new() -> Self {
        Self {
            scalar_scanner: ScannerScalar::new(),
        }
    }
    
    pub fn get_instance() -> &'static ScannerScalarIterative {
        static mut INSTANCE: Option<ScannerScalarIterative> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = ScannerScalarIterative::new();
                INSTANCE = Some(instance);
            });

            return INSTANCE.as_ref().unwrap_unchecked();
        }
    }
}

/// Implements a scalar (ie CPU bound, non-SIMD) region scanning algorithm. This simply iterates over a region of memory,
/// comparing each element based on the provided constraints. Elements that pass the constraint are grouped in sub-regions and returned.
impl Scanner for ScannerScalarIterative {

    /// Performs a sequential iteration over a region of memory, performing the scan comparison. A run-length encoding algorithm
    /// is used to generate new sub-regions as the scan progresses.
    fn scan_region(&self, snapshot_region: &SnapshotRegion, snapshot_sub_region: &SnapshotSubRegion, constraint: &ScanConstraint) -> Vec<SnapshotSubRegion> {
        let mut run_length_encoder = SnapshotSubRegionRunLengthEncoder::new(snapshot_sub_region);
        let mut current_value_pointer = snapshot_region.get_sub_region_current_values_pointer(&snapshot_sub_region);
        let mut previous_value_pointer = snapshot_region.get_sub_region_previous_values_pointer(&snapshot_sub_region);
        let data_type = constraint.get_element_type();
        let constraint_value = constraint.get_constraint_value().unwrap();
        let mut current_value = constraint_value.clone();
        let mut previous_value = constraint_value.clone();
        let current_value = current_value.borrow_mut();
        let previous_value = previous_value.borrow_mut();
        let aligned_element_count = snapshot_sub_region.get_element_count(constraint.get_alignment(), data_type.size_in_bytes());

        for _ in 0..aligned_element_count {
            if self.scalar_scanner.do_compare_action(current_value_pointer, previous_value_pointer, current_value, previous_value, &constraint) {
                run_length_encoder.encode_range(constraint.get_alignment() as u64);
            } else {
                run_length_encoder.finalize_current_encode_unchecked(constraint.get_alignment() as u64, data_type.size_in_bytes());
            }

            unsafe {
                current_value_pointer = current_value_pointer.add(constraint.get_alignment() as usize);
                previous_value_pointer = previous_value_pointer.add(constraint.get_alignment() as usize);
            }
        }

        run_length_encoder.finalize_current_encode_unchecked(0, data_type.size_in_bytes());

        return run_length_encoder.get_collected_regions().to_owned();
    }
}
