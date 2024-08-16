use crate::scanners::comparers::vector::scanner_vector::SnapshotElementScannerVector;
use crate::scanners::constraints::operation_constraint::ScanConstraint;
use crate::snapshots::snapshot_sub_region::NormalizedRegion;
use std::simd::u8x16;
use std::sync::Arc;
use std::sync::RwLock;

pub struct SnapshotRegionVectorScannerAligned {
    base_scanner: SnapshotElementScannerVector,
}

impl<'a> SnapshotRegionVectorScannerAligned {
    pub fn new() -> Self {
        Self {
            base_scanner: SnapshotElementScannerVector::new(),
        }
    }

    fn scan_region(&self, snapshot_sub_region: &Arc<RwLock<NormalizedRegion>>, constraint: &ScanConstraint) -> Vec<NormalizedRegion> {
        self.base_scanner.initialize(snapshot_sub_region, constraints);
        let vector_comparer = self.base_scanner.vector_compare_func.take().unwrap();
        self.base_scanner.perform_vector_scan(u8x16::splat(0), 16, vector_comparer)
    }
    
}
