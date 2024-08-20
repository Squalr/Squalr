use squalr_engine_common::dynamic_struct::data_type::DataType;

use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::comparers::scalar::scanner_scalar_encoder::ScannerScalarEncoder;
use crate::scanners::comparers::snapshot_scanner::Scanner;
use crate::scanners::constraints::scan_constraint::{ScanConstraint, ScanFilterConstraint};
use crate::snapshots::snapshot_region::SnapshotRegion;
use std::sync::Once;

pub struct ScannerScalarIterative {
}

impl ScannerScalarIterative {
    fn new(
    ) -> Self {
        Self { }
    }
    
    pub fn get_instance(
    ) -> &'static ScannerScalarIterative {
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
    fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_constraint: &ScanConstraint,
        filter_constraint: &ScanFilterConstraint,
    ) -> Vec<SnapshotRegionFilter> {
        let data_type = filter_constraint.get_data_type();
        let data_type_size = data_type.size_in_bytes();
        let memory_alignment = filter_constraint.get_memory_alignment_or_default(data_type);
        let aligned_element_count = snapshot_region_filter.get_element_count(memory_alignment, data_type_size);
        let encoder = ScannerScalarEncoder::get_instance();
        let current_value_pointer = snapshot_region.get_current_values_pointer(&snapshot_region_filter);
        let previous_value_pointer = snapshot_region.get_previous_values_pointer(&snapshot_region_filter);

        let results = encoder.encode(
            current_value_pointer,
            previous_value_pointer,
            scan_constraint,
            filter_constraint,
            snapshot_region_filter.get_base_address(),
            aligned_element_count
        );
        
        // TODO: Boundary merging on adjacent regions

        return results;
    }
}
