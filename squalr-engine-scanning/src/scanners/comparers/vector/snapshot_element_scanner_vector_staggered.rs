use crate::scanners::comparers::vector::snapshot_element_scanner_vector::SnapshotElementScannerVector;
use crate::scanners::constraints::scan_constraints::ScanConstraints;
use crate::snapshots::snapshot_element_range::SnapshotElementRange;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::collections::HashMap;
use std::simd::{u16x8, u8x16, u32x4, u64x2};
use std::simd::num::SimdUint;

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
                u16x8::splat(0x00FF).cast(),
                u16x8::splat(0xFF00).cast(),
            ]);
            map
        });

        // Data type size 4
        staggered_mask_map.insert(4, {
            let mut map = HashMap::new();
            map.insert(MemoryAlignment::Alignment1, vec![
                u32x4::splat(0x000000FF).cast(),
                u32x4::splat(0x0000FF00).cast(),
                u32x4::splat(0x00FF0000).cast(),
                u32x4::splat(0xFF000000).cast(),
            ]);
            map.insert(MemoryAlignment::Alignment2, vec![
                u32x4::splat(0x0000FFFF).cast(),
                u32x4::splat(0xFFFF0000).cast(),
            ]);
            map
        });

        // Data type size 8
        staggered_mask_map.insert(8, {
            let mut map = HashMap::new();
            map.insert(MemoryAlignment::Alignment1, vec![
                u64x2::splat(0x00000000000000FF).cast(),
                u64x2::splat(0x000000000000FF00).cast(),
                u64x2::splat(0x0000000000FF0000).cast(),
                u64x2::splat(0x00000000FF000000).cast(),
                u64x2::splat(0x000000FF00000000).cast(),
                u64x2::splat(0x0000FF0000000000).cast(),
                u64x2::splat(0x00FF000000000000).cast(),
                u64x2::splat(0xFF00000000000000).cast(),
            ]);
            map.insert(MemoryAlignment::Alignment2, vec![
                u64x2::splat(0x000000000000FFFF).cast(),
                u64x2::splat(0x00000000FFFF0000).cast(),
                u64x2::splat(0x0000FFFF00000000).cast(),
                u64x2::splat(0xFFFF000000000000).cast(),
            ]);
            map.insert(MemoryAlignment::Alignment4, vec![
                u64x2::splat(0x00000000FFFFFFFF).cast(),
                u64x2::splat(0xFFFFFFFF00000000).cast(),
            ]);
            map
        });

        Self {
            base_scanner: SnapshotElementScannerVector::new(),
            staggered_mask_map,
        }
    }

    pub fn scan_region(
        &mut self,
        element_range: SnapshotElementRange,
        constraints: &ScanConstraints,
    ) -> Vec<SnapshotElementRange> {
        self.base_scanner.initialize(&element_range.clone(), constraints);
        let scan_count_per_vector = self.base_scanner.base_scanner.get_data_type_size() / self.base_scanner.base_scanner.get_alignment() as usize;
        let offset_vector_increment_size = 16 - (self.base_scanner.base_scanner.get_alignment() as usize * scan_count_per_vector);
        let staggered_masks = self.staggered_mask_map.get(&(self.base_scanner.base_scanner.get_data_type_size() as i32)).unwrap()
            .get(&self.base_scanner.base_scanner.get_alignment()).unwrap();

        self.base_scanner.perform_vector_scan(u8x16::splat(0), offset_vector_increment_size, &move || {
            self.staggered_vector_scan(staggered_masks)
        })
    }

    fn staggered_vector_scan(&self, staggered_masks: &[u8x16]) -> u8x16 {
        let scan_count_per_vector = self.base_scanner.base_scanner.get_data_type_size() / self.base_scanner.base_scanner.get_alignment() as usize;
        let mut run_length_encoded_scan_result = u8x16::splat(0);

        for alignment_offset in 0..scan_count_per_vector {
            run_length_encoded_scan_result = run_length_encoded_scan_result.bitor(self.base_scanner.vector_compare_func.as_ref().unwrap()().bitand(staggered_masks[alignment_offset]));

            self.base_scanner.vector_read_offset += self.base_scanner.base_scanner.get_alignment() as usize;

            if self.base_scanner.vector_read_offset >= self.base_scanner.base_scanner.element_range.as_ref().unwrap().range - 16 {
                self.base_scanner.vector_read_offset += self.base_scanner.base_scanner.get_alignment() as usize * (scan_count_per_vector - alignment_offset - 1);
                break;
            }
        }

        return run_length_encoded_scan_result;
    }
}
