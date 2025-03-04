use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::encoders::scalar::scanner_scalar_encoder::ScannerScalarEncoder;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::scanners::snapshot_scanner::Scanner;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_common::structures::data_types::data_type::DataType;
use squalr_engine_common::structures::memory_alignment::MemoryAlignment;

pub struct ScannerScalarIterative {}

impl ScannerScalarIterative {
    pub fn new() -> Self {
        Self {}
    }
}

/// Implements a scalar (ie CPU bound, non-SIMD) region scanning algorithm. This simply iterates over a region of memory,
/// comparing each element based on the provided parameters. Elements that pass the scan are grouped into filter ranges and returned.
impl Scanner for ScannerScalarIterative {
    /// Performs a sequential iteration over a region of memory, performing the scan comparison. A run-length encoding algorithm
    /// is used to generate new sub-regions as the scan progresses.
    fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_parameters: &ScanParameters,
        data_type: &Box<dyn DataType>,
        memory_alignment: MemoryAlignment,
    ) -> Vec<SnapshotRegionFilter> {
        let results = ScannerScalarEncoder::scalar_encode(
            snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter),
            snapshot_region.get_previous_values_filter_pointer(&snapshot_region_filter),
            scan_parameters,
            data_type,
            memory_alignment,
            snapshot_region_filter.get_base_address(),
            snapshot_region_filter.get_element_count(data_type, memory_alignment),
        );

        results
    }
}
