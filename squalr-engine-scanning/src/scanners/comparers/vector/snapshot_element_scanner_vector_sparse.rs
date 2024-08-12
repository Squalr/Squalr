use crate::scanners::constraints::scan_constraints::ScanConstraints;
use crate::snapshots::snapshot_element_range::SnapshotElementRange;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::collections::HashMap;
use std::simd::{u8x16, u16x8, u32x4, u64x2};

pub struct SnapshotRegionScannerVectorSparse {
    base_scanner: SnapshotRegionScannerVector,
    sparse_masks: HashMap<MemoryAlignment, u8x16>,
}

impl SnapshotRegionScannerVectorSparse {
    pub fn new() -> Self {
        let mut sparse_masks = HashMap::new();
        sparse_masks.insert(MemoryAlignment::Alignment1, u8x16::splat(0xFF));
        sparse_masks.insert(MemoryAlignment::Alignment2, u16x8::splat(0xFF00).cast());
        sparse_masks.insert(MemoryAlignment::Alignment4, u32x4::splat(0xFF000000).cast());
        sparse_masks.insert(MemoryAlignment::Alignment8, u64x2::splat(0xFF00000000000000).cast());

        Self {
            base_scanner: SnapshotRegionScannerVector::new(),
            sparse_masks,
        }
    }

    pub fn scan_region(
        &mut self,
        element_range: SnapshotElementRange,
        constraints: &ScanConstraints,
    ) -> Vec<SnapshotElementRange> {
        self.base_scanner.initialize(element_range.clone(), constraints);
        let sparse_mask = self.sparse_masks.get(&self.base_scanner.base_scanner.alignment).unwrap();
        self.base_scanner.perform_vector_scan(*sparse_mask, 16, &self.base_scanner.vector_compare_func.as_ref().unwrap())
    }
}
