use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::encoders::scalar::scanner_scalar_encoder::ScannerScalarEncoder;
use crate::scanners::snapshot_scanner::Scanner;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_api::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use squalr_engine_api::structures::scanning::scan_parameters_local::ScanParametersLocal;

pub struct ScannerScalarIterative {}

impl ScannerScalarIterative {}

/// Implements a scalar (ie CPU bound, non-SIMD) region scanning algorithm. This simply iterates over a region of memory,
/// comparing each element based on the provided parameters. Elements that pass the scan are grouped into filter ranges and returned.
impl Scanner for ScannerScalarIterative {
    /// Performs a sequential iteration over a region of memory, performing the scan comparison. A run-length encoding algorithm
    /// is used to generate new sub-regions as the scan progresses.
    fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Vec<SnapshotRegionFilter> {
        let results = ScannerScalarEncoder::scalar_encode(
            snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter),
            snapshot_region.get_previous_values_filter_pointer(&snapshot_region_filter),
            scan_parameters_global,
            scan_parameters_local,
            snapshot_region_filter.get_base_address(),
            snapshot_region_filter.get_element_count(scan_parameters_local.get_data_type(), scan_parameters_local.get_memory_alignment_or_default()),
        );

        results
    }
}
