use crate::results::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::encoders::scalar::scanner_scalar_encoder_byte_array::ScannerScalarEncoderByteArray;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::scanners::snapshot_scanner::Scanner;
use crate::snapshots::snapshot_region::SnapshotRegion;
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

            return INSTANCE.as_ref().unwrap_unchecked();
        }
    }
}

/// Implements a scalar (ie CPU bound, non-SIMD) array of bytes region scanning algorithm. This works by using the Boyer-Moore
/// algorithm to encode matches as they are discovered.
impl Scanner for ScannerScalarIterativeByteArray {
    /// Performs a sequential iteration over a region of memory, performing the scan comparison. A run-length encoding algorithm
    /// is used to generate new sub-regions as the scan progresses.
    unsafe fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_parameters: &ScanParameters,
        scan_filter_parameters: &ScanFilterParameters,
    ) -> Vec<SnapshotRegionFilter> {
        let encoder = ScannerScalarEncoderByteArray::get_instance();

        let results = encoder.encode(
            snapshot_region.get_current_values_pointer(&snapshot_region_filter),
            snapshot_region.get_previous_values_pointer(&snapshot_region_filter),
            scan_parameters,
            scan_filter_parameters,
            snapshot_region_filter.get_base_address(),
            snapshot_region_filter.get_region_size(),
        );

        return results;
    }
}
