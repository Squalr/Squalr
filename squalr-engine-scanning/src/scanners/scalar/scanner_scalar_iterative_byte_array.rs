use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::encoders::scalar::scanner_scalar_encoder_byte_array::ScannerScalarEncoderByteArray;
use crate::scanners::snapshot_scanner::Scanner;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_api::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use squalr_engine_api::structures::scanning::scan_parameters_local::ScanParametersLocal;
use std::sync::Once;

pub struct ScannerScalarIterativeByteArray {}

impl ScannerScalarIterativeByteArray {
    fn new() -> Self {
        Self {}
    }

    pub fn get_instance() -> &'static ScannerScalarIterativeByteArray {
        static mut INSTANCE: Option<ScannerScalarIterativeByteArray> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = ScannerScalarIterativeByteArray::new();
                INSTANCE = Some(instance);
            });

            #[allow(static_mut_refs)]
            INSTANCE.as_ref().unwrap_unchecked()
        }
    }
}

/// Implements a scalar (ie CPU bound, non-SIMD) array of bytes region scanning algorithm. This works by using the Boyer-Moore
/// algorithm to encode matches as they are discovered.
impl Scanner for ScannerScalarIterativeByteArray {
    /// Performs a sequential iteration over a region of memory, performing the scan comparison. A run-length encoding algorithm
    /// is used to generate new sub-regions as the scan progresses.
    fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Vec<SnapshotRegionFilter> {
        let results = ScannerScalarEncoderByteArray::scalar_encode_byte_array(
            snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter),
            snapshot_region.get_previous_values_filter_pointer(&snapshot_region_filter),
            scan_parameters_global,
            scan_parameters_local,
            snapshot_region_filter.get_base_address(),
            snapshot_region_filter.get_region_size(),
        );

        results
    }
}
