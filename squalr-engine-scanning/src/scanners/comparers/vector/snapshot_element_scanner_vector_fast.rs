use crate::scanners::comparers::vector::snapshot_element_scanner_vector::SnapshotElementScannerVector;
use crate::scanners::constraints::scan_constraints::ScanConstraints;
use crate::snapshots::snapshot_element_range::SnapshotElementRange;
use std::simd::u8x16;

pub struct SnapshotRegionVectorScannerFast<'a> {
    base_scanner: SnapshotElementScannerVector<'a>,
}

impl<'a> SnapshotRegionVectorScannerFast<'a> {
    pub fn new() -> Self {
        Self {
            base_scanner: SnapshotElementScannerVector::new(),
        }
    }

    pub fn scan_region(
        &mut self,
        element_range: &'a SnapshotElementRange<'a>,
        constraints: &'a ScanConstraints,
    ) -> Vec<SnapshotElementRange<'a>> {
        self.base_scanner.initialize(element_range, constraints);
        let vector_comparer = self.base_scanner.vector_compare_func.take().unwrap();
        self.base_scanner.perform_vector_scan(u8x16::splat(0), 16, vector_comparer)
    }
    
}
