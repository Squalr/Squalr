use crate::scanners::comparers::vector::snapshot_element_scanner_vector::SnapshotElementScannerVector;
use crate::scanners::constraints::scan_constraints::ScanConstraints;
use crate::snapshots::snapshot_sub_region::SnapshotSubRegion;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::collections::HashMap;
use std::simd::{ToBytes, u8x16, u16x8, u32x4, u64x2};

pub struct SnapshotElementScannerVectorSparse<'a> {
    base_scanner: SnapshotElementScannerVector<'a>,
    sparse_masks: HashMap<MemoryAlignment, u8x16>,
}

impl<'a> SnapshotElementScannerVectorSparse<'a> {
    pub fn new() -> Self {
        let mut sparse_masks = HashMap::new();

        // Store masks as u8x16 by converting larger types
        sparse_masks.insert(MemoryAlignment::Alignment1, u8x16::splat(0xFF));
        sparse_masks.insert(
            MemoryAlignment::Alignment2,
            u16x8::splat(0xFF00).to_le_bytes().into(),
        );
        sparse_masks.insert(
            MemoryAlignment::Alignment4,
            u32x4::splat(0xFF000000).to_le_bytes().into(),
        );
        sparse_masks.insert(
            MemoryAlignment::Alignment8,
            u64x2::splat(0xFF00000000000000).to_le_bytes().into(),
        );

        Self {
            base_scanner: SnapshotElementScannerVector::new(),
            sparse_masks,
        }
    }

    pub fn scan_region(
        &mut self,
        snapshot_sub_region: &'a SnapshotSubRegion<'a>,
        constraints: &'a ScanConstraints,
    ) -> Vec<SnapshotSubRegion<'a>> {
        self.base_scanner.initialize(snapshot_sub_region, constraints);

        let sparse_mask = *self
            .sparse_masks
            .get(&self.base_scanner.base_scanner.get_byte_alignment())
            .unwrap();

        let vector_comparer = self.base_scanner.vector_compare_func.take().unwrap();
        self.base_scanner.perform_vector_scan(sparse_mask, 16, vector_comparer)
    }
}
