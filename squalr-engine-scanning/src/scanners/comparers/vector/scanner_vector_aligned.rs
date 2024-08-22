use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::comparers::snapshot_scanner::Scanner;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::snapshots::snapshot_region::SnapshotRegion;
use std::sync::Once;

use super::encoder::scanner_vector_encoder_128::ScannerVectorEncoder;

pub struct ScannerVectorAligned {
}

impl ScannerVectorAligned {
    fn new(
    ) -> Self {
        Self { }
    }
    
    pub fn get_instance(
    ) -> &'static ScannerVectorAligned {
        static mut INSTANCE: Option<ScannerVectorAligned> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = ScannerVectorAligned::new();
                INSTANCE = Some(instance);
            });

            return INSTANCE.as_ref().unwrap_unchecked();
        }
    }
}

/// Implements a scalar (ie CPU bound, non-SIMD) region scanning algorithm. This simply iterates over a region of memory,
/// comparing each element based on the provided parameters. Elements that pass the scan are grouped into filter ranges and returned.
impl Scanner for ScannerVectorAligned {

    /// Performs a sequential iteration over a region of memory, performing the scan comparison. A run-length encoding algorithm
    /// is used to generate new sub-regions as the scan progresses.
    fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_parameters: &ScanParameters,
        scan_filter_parameters: &ScanFilterParameters,
    ) -> Vec<SnapshotRegionFilter> {
        let data_type = scan_filter_parameters.get_data_type();
        let data_type_size = data_type.size_in_bytes();
        let memory_alignment = scan_filter_parameters.get_memory_alignment_or_default();
        let aligned_element_count = snapshot_region_filter.get_element_count(memory_alignment, data_type_size);
        let encoder = ScannerVectorEncoder::get_instance();
        let current_value_pointer = snapshot_region.get_current_values_pointer(&snapshot_region_filter);
        let previous_value_pointer = snapshot_region.get_previous_values_pointer(&snapshot_region_filter);

        let results = encoder.encode(
            current_value_pointer,
            previous_value_pointer,
            scan_parameters,
            scan_filter_parameters,
            snapshot_region_filter.get_base_address(),
            aligned_element_count
        );

        return results;
    }
}
