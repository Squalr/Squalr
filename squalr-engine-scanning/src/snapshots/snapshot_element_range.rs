use crate::snapshots::snapshot_element_indexer::SnapshotElementIndexer;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::cell::RefCell;
use std::iter::Iterator;

#[derive(Debug, Clone)]
pub struct SnapshotElementRange<'a> {
    pub parent_region: &'a RefCell<SnapshotRegion>,
    pub region_offset: usize,
    pub range: usize,
}

impl<'a> SnapshotElementRange<'a> {
    pub fn new(parent_region: &'a RefCell<SnapshotRegion>) -> Self { // Adjust to accept a RefCell
        Self::with_offset_and_range(parent_region, 0, parent_region.borrow().get_region_size() as usize)
    }

    pub fn with_offset_and_range(parent_region: &'a RefCell<SnapshotRegion>, region_offset: usize, range: usize) -> Self {
        Self {
            parent_region,
            region_offset,
            range,
        }
    }
    
    pub fn set_current_values(&mut self, values: Vec<u8>) {
        self.parent_region.borrow_mut().set_current_values(values);
    }

    pub fn get_current_values(&self) -> Vec<u8> {
        self.parent_region.borrow().current_values.clone()
    }

    pub fn get_base_element_address(&self) -> u64 {
        return self.parent_region.borrow().get_base_address() + self.region_offset as u64;
    }

    pub fn get_end_element_address(&self) -> u64 {
        self.get_base_element_address() + self.range as u64
    }

    pub fn get_byte_count(&self, data_type_size: usize) -> usize {
        let desired_spill_over_bytes = if data_type_size > 1 { data_type_size - 1 } else { 0 };
        let available_spill_over_bytes = (self.parent_region.borrow().get_base_address() + self.range as u64) - self.get_end_element_address();
        let used_spill_over_bytes = std::cmp::min(desired_spill_over_bytes, available_spill_over_bytes as usize);

        self.range + used_spill_over_bytes
    }

    pub fn get_aligned_element_count(&self, alignment: MemoryAlignment) -> usize {
        let alignment_value = alignment as usize;
        self.range / if alignment_value == 0 { 1 } else { alignment_value }
    }

    pub fn resize_for_safe_reading(&mut self, data_type_size: usize) {
        let parent_region_size = self.parent_region.borrow().get_region_size() as usize;
        self.range = std::cmp::min(self.range, parent_region_size - self.region_offset - data_type_size);
    }

    pub fn get_element_indexer(&self, index: usize, alignment: MemoryAlignment) -> SnapshotElementIndexer {
        SnapshotElementIndexer::new(self, alignment, index)
    }

    pub fn iterate_elements(&self, alignment: MemoryAlignment) -> SnapshotElementIterator {
        SnapshotElementIterator::new(self, alignment)
    }
}

pub struct SnapshotElementIterator<'a> {
    element_range: &'a SnapshotElementRange<'a>,
    alignment: MemoryAlignment,
    current_index: usize,
    total_elements: usize,
}

impl<'a> SnapshotElementIterator<'a> {
    pub fn new(element_range: &'a SnapshotElementRange<'a>, alignment: MemoryAlignment) -> Self {
        let total_elements = element_range.get_aligned_element_count(alignment);
        Self {
            element_range,
            alignment,
            current_index: 0,
            total_elements,
        }
    }
}

impl<'a> Iterator for SnapshotElementIterator<'a> {
    type Item = SnapshotElementIndexer<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index < self.total_elements {
            let indexer = self.element_range.get_element_indexer(self.current_index, self.alignment);
            self.current_index += 1;
            Some(indexer)
        } else {
            None
        }
    }
}