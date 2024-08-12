use crate::scanners::comparers::vector::snapshot_element_scanner_vector::SnapshotElementScannerVector;
use crate::scanners::constraints::scan_constraints::ScanConstraints;
use crate::snapshots::snapshot_element_range::SnapshotElementRange;
use std::simd::u8x16;

pub struct SnapshotRegionVectorScannerFast {
    base_scanner: SnapshotElementScannerVector,
}

impl SnapshotRegionVectorScannerFast {
    pub fn new() -> Self {
        Self {
            base_scanner: SnapshotElementScannerVector::new(),
        }
    }

    pub fn scan_region(
        &mut self,
        element_range: SnapshotElementRange,
        constraints: &ScanConstraints,
    ) -> Vec<SnapshotElementRange> {
        self.base_scanner.initialize(&element_range.clone(), constraints);
        self.base_scanner.perform_vector_scan(u8x16::splat(0), 16, &self.base_scanner.vector_compare_func.as_ref().unwrap())
    }
}
