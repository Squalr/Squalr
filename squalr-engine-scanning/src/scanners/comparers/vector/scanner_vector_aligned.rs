use crate::scanners::comparers::vector::snapshot_element_scanner_vector::SnapshotElementScannerVector;
use crate::scanners::constraints::scan_constraints::ScanConstraints;
use crate::snapshots::snapshot_sub_region::SnapshotSubRegion;
use std::simd::u8x16;

pub struct SnapshotRegionVectorScannerAligned<'a> {
    base_scanner: SnapshotElementScannerVector<'a>,
}

impl<'a> SnapshotRegionVectorScannerAligned<'a> {
    pub fn new() -> Self {
        Self {
            base_scanner: SnapshotElementScannerVector::new(),
        }
    }

    pub fn scan_region(
        &mut self,
        snapshot_sub_region: &'a SnapshotSubRegion<'a>,
        constraints: &'a ScanConstraints,
    ) -> Vec<SnapshotSubRegion<'a>> {
        self.base_scanner.initialize(snapshot_sub_region, constraints);
        let vector_comparer = self.base_scanner.vector_compare_func.take().unwrap();
        self.base_scanner.perform_vector_scan(u8x16::splat(0), 16, vector_comparer)
    }
    
}
