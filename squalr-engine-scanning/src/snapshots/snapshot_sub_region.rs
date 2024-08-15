use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::cmp::max;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct SnapshotSubRegion {
    pub parent_region: Arc<RwLock<SnapshotRegion>>,
    region_offset: usize,
    range: usize,
}

impl SnapshotSubRegion {
    pub fn new(parent_region: Arc<RwLock<SnapshotRegion>>) -> Self {
        let parent_region_size = parent_region.read().unwrap().get_region_size() as usize;
        Self::new_with_offset_and_range(parent_region, 0, parent_region_size)
    }

    pub fn new_with_offset_and_range(parent_region: Arc<RwLock<SnapshotRegion>>, region_offset: usize, range: usize) -> Self {
        Self {
            parent_region,
            region_offset,
            range,
        }
    }
    
    pub fn get_current_values_pointer(&self) -> *const u8 {
        let parent_region = self.parent_region.read().unwrap();
        let current_values = parent_region.get_current_values();
        unsafe { current_values.as_ptr().add(self.region_offset) }
    }

    pub fn get_previous_values_pointer(&self) -> *const u8 {
        let parent_region = self.parent_region.read().unwrap();
        let previous_values = parent_region.get_previous_values();
        unsafe { previous_values.as_ptr().add(self.region_offset) }
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

    pub fn get_byte_count(&self) -> u64 {
        return self.range as u64;
    }

    pub fn get_byte_count_for_data_type_size(&self, data_type_size: usize) -> u64 {
        if data_type_size > self.range {
            return 0;
        } else {
            return (self.range - data_type_size) as u64;
        }
    }

    pub fn get_element_count(&self, data_type_size: usize, alignment: MemoryAlignment) -> u64 {
        let alignment = max(alignment as u64, 1);
        let misalignment = self.get_misalignment(alignment);

        if misalignment > self.range as u64 {
            return 0;
        }

        let effective_range = self.range as u64 - misalignment;
        let byte_count = effective_range.saturating_sub(data_type_size as u64);

        return byte_count / alignment;
    }

    pub fn get_misalignment(&self, alignment: u64) -> u64 {
        let base_address = self.get_base_element_address();
        let aligned_base = (base_address + alignment - 1) / alignment * alignment;

        return aligned_base - base_address;
    }
}
