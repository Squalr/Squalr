use crate::scanners::comparers::vector::scanner_vector::SnapshotElementScannerVector;
use crate::scanners::constraints::scan_constraint::ScanConstraint;
use crate::snapshots::snapshot_sub_region::SnapshotSubRegion;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::collections::HashMap;
use std::ops::BitAnd;
use std::simd::{ToBytes, u8x16, u16x8, u32x4, u64x2};
use std::sync::{Arc, RwLock};

pub struct SnapshotRegionScannerVectorStaggered {
    base_scanner: SnapshotElementScannerVector,
    staggered_mask_map: HashMap<i32, HashMap<MemoryAlignment, Vec<u8x16>>>,
}

impl SnapshotRegionScannerVectorStaggered {
    pub fn new() -> Self {
        let mut staggered_mask_map = HashMap::new();

        // Data type size 2
        staggered_mask_map.insert(2, {
            let mut map = HashMap::new();
            map.insert(MemoryAlignment::Alignment1, vec![
                u16x8::splat(0x00FF).to_le_bytes().into(),
                u16x8::splat(0xFF00).to_le_bytes().into(),
            ]);
            map
        });

        // Data type size 4
        staggered_mask_map.insert(4, {
            let mut map = HashMap::new();
            map.insert(MemoryAlignment::Alignment1, vec![
                u32x4::splat(0x000000FF).to_le_bytes().into(),
                u32x4::splat(0x0000FF00).to_le_bytes().into(),
                u32x4::splat(0x00FF0000).to_le_bytes().into(),
                u32x4::splat(0xFF000000).to_le_bytes().into(),
            ]);
            map.insert(MemoryAlignment::Alignment2, vec![
                u32x4::splat(0x0000FFFF).to_le_bytes().into(),
                u32x4::splat(0xFFFF0000).to_le_bytes().into(),
            ]);
            map
        });

        // Data type size 8
        staggered_mask_map.insert(8, {
            let mut map = HashMap::new();
            map.insert(MemoryAlignment::Alignment1, vec![
                u64x2::splat(0x00000000000000FF).to_le_bytes().into(),
                u64x2::splat(0x000000000000FF00).to_le_bytes().into(),
                u64x2::splat(0x0000000000FF0000).to_le_bytes().into(),
                u64x2::splat(0x00000000FF000000).to_le_bytes().into(),
                u64x2::splat(0x000000FF00000000).to_le_bytes().into(),
                u64x2::splat(0x0000FF0000000000).to_le_bytes().into(),
                u64x2::splat(0x00FF000000000000).to_le_bytes().into(),
                u64x2::splat(0xFF00000000000000).to_le_bytes().into(),
            ]);
            map.insert(MemoryAlignment::Alignment2, vec![
                u64x2::splat(0x000000000000FFFF).to_le_bytes().into(),
                u64x2::splat(0x00000000FFFF0000).to_le_bytes().into(),
                u64x2::splat(0x0000FFFF00000000).to_le_bytes().into(),
                u64x2::splat(0xFFFF000000000000).to_le_bytes().into(),
            ]);
            map.insert(MemoryAlignment::Alignment4, vec![
                u64x2::splat(0x00000000FFFFFFFF).to_le_bytes().into(),
                u64x2::splat(0xFFFFFFFF00000000).to_le_bytes().into(),
            ]);
            map
        });

        Self {
            base_scanner: SnapshotElementScannerVector::new(),
            staggered_mask_map,
        }
    }

    fn scan_region(&self, snapshot_sub_region: &Arc<RwLock<SnapshotSubRegion>>, constraint: &ScanConstraint) -> Vec<Arc<RwLock<SnapshotSubRegion>>> {
        self.base_scanner.initialize(snapshot_sub_region, constraint);

        let data_type_size = self.base_scanner.base_scanner.get_data_type_size();
        let alignment = self.base_scanner.base_scanner.get_alignment();

        let staggered_masks = self.staggered_mask_map
            .get(&(data_type_size as i32))
            .unwrap()
            .get(&alignment)
            .unwrap()
            .clone();

        let offset_vector_increment_size = 16 - (alignment as usize * (data_type_size / alignment as usize));

        if let Some(vector_comparer) = self.base_scanner.vector_compare_func.take() {
            let comparer_result = vector_comparer();
            return self.base_scanner.perform_vector_scan(u8x16::splat(0), offset_vector_increment_size, Box::new(move || {
                SnapshotRegionScannerVectorStaggered::staggered_vector_scan(
                    staggered_masks.clone(),
                    comparer_result,
                    alignment,
                    data_type_size
                )
            }));
        } else {
            return Vec::new();
        }
    }

    fn staggered_vector_scan(
        staggered_masks: Vec<u8x16>,
        comparer_result: u8x16,
        alignment: MemoryAlignment,
        data_type_size: usize,
    ) -> u8x16 {
        let mut result = u8x16::splat(0);
        let mut vector_read_offset = 0;

        for (i, mask) in staggered_masks.iter().enumerate() {
            result |= comparer_result.bitand(*mask);
            vector_read_offset += alignment as usize;

            if vector_read_offset >= data_type_size - 16 {
                vector_read_offset += alignment as usize * (staggered_masks.len() - i - 1);
                break;
            }
        }

        return result;
    }
}
