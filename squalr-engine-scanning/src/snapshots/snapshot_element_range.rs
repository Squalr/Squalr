use crate::snapshots::snapshot_element_indexer::SnapshotElementIndexer;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::iter::Iterator;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct SnapshotElementRange {
    pub parent_region: Arc<RwLock<SnapshotRegion>>,
    region_offset: usize,
    range: usize,
}

impl SnapshotElementRange {
    pub fn new(parent_region: Arc<RwLock<SnapshotRegion>>) -> Self {
        let parent_region_size = parent_region.read().unwrap().get_region_size() as usize;
        Self::with_offset_and_range(parent_region, 0, parent_region_size)
    }

    pub fn with_offset_and_range(parent_region: Arc<RwLock<SnapshotRegion>>, region_offset: usize, range: usize) -> Self {
        Self {
            parent_region,
            region_offset,
            range,
        }
    }

    pub fn get_base_element_address(&self) -> u64 {
        return self.parent_region.read().unwrap().get_base_address() + self.region_offset as u64;
    }

    pub fn get_end_element_address(&self) -> u64 {
        return self.get_base_element_address() + self.range as u64;
    }

    pub fn get_region_offset(&self) -> usize {
        return self.region_offset;
    }
    
    pub fn get_range(&self) -> usize {
        return self.range;
    }

    pub fn get_byte_count(&self, data_type_size: usize) -> usize {
        let desired_spill_over_bytes = if data_type_size > 1 { data_type_size - 1 } else { 0 };
        let available_spill_over_bytes = (self.parent_region.read().unwrap().get_base_address() + self.range as u64) - self.get_end_element_address();
        let used_spill_over_bytes = std::cmp::min(desired_spill_over_bytes, available_spill_over_bytes as usize);

        return self.range + used_spill_over_bytes;
    }

    pub fn get_aligned_element_count(&self, alignment: MemoryAlignment) -> usize {
        let alignment_value = alignment as usize;

        return self.range / if alignment_value == 0 { 1 } else { alignment_value };
    }

    pub fn resize_for_safe_reading(&mut self, data_type_size: usize) {
        let parent_region_size = self.parent_region.read().unwrap().get_region_size() as usize;
        self.range = std::cmp::min(self.range, parent_region_size - self.region_offset - data_type_size);
    }

    pub fn get_element_indexer(&self, index: usize, alignment: MemoryAlignment) -> SnapshotElementIndexer {
        return SnapshotElementIndexer::new(self.clone().into(), alignment, index);
    }

    pub fn iterate_elements(&self, alignment: MemoryAlignment) -> SnapshotElementIterator {
        return SnapshotElementIterator::new(self.clone().into(), alignment);
    }
}

pub struct SnapshotElementIterator {
    element_range: Arc<SnapshotElementRange>,
    alignment: MemoryAlignment,
    current_index: usize,
    total_elements: usize,
}

impl SnapshotElementIterator {
    pub fn new(element_range: Arc<SnapshotElementRange>, alignment: MemoryAlignment) -> Self {
        let total_elements = element_range.get_aligned_element_count(alignment);
        Self {
            element_range,
            alignment,
            current_index: 0,
            total_elements,
        }
    }
}

impl Iterator for SnapshotElementIterator {
    type Item = SnapshotElementIndexer;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index < self.total_elements {
            let indexer = self.element_range.get_element_indexer(self.current_index, self.alignment);
            self.current_index += 1;

            return Some(indexer);
        } else {
            return None;
        }
    }
}
