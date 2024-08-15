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

/// Implements a scalar (ie CPU bound, non-SIMD) region scanning algorithm. This simply iterates over a region of memory,
/// comparing each element based on the provided constraints. Elements that pass the constraint are grouped in sub-regions and returned.
impl Scanner for ScannerScalarIterative {
    fn scan_region(&self, snapshot_sub_region: &Arc<RwLock<SnapshotSubRegion>>, constraint: &ScanConstraint) -> Vec<Arc<RwLock<SnapshotSubRegion>>> {
        let mut run_length_encoder = SnapshotSubRegionRunLengthEncoder::new(snapshot_sub_region.clone());
        run_length_encoder.initialize();

        let snapshot_sub_region = snapshot_sub_region.read().unwrap();
        let mut current_value_pointer = snapshot_sub_region.get_current_values_pointer();
        let mut previous_value_pointer = snapshot_sub_region.get_previous_values_pointer();
        let data_type = constraint.get_element_type();
        let aligned_element_count = snapshot_sub_region.get_element_count(constraint.get_alignment(), data_type.size_in_bytes());

        for _ in 0..aligned_element_count {
            if self.scalar_scanner.do_compare_action(current_value_pointer, previous_value_pointer, &constraint, &data_type) {
                run_length_encoder.encode_range(constraint.get_alignment() as usize);
            } else {
                run_length_encoder.finalize_current_encode_unchecked(constraint.get_alignment() as usize, data_type.size_in_bytes());
            }

            unsafe {
                current_value_pointer = current_value_pointer.add(constraint.get_alignment() as usize);
                previous_value_pointer = previous_value_pointer.add(constraint.get_alignment() as usize);
            }
        }

        run_length_encoder.finalize_current_encode_unchecked(0, data_type.size_in_bytes());

        return run_length_encoder.get_collected_regions().clone();
    }
}
