use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_architecture::vectors::vectors;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::cmp::max;

#[derive(Debug, Clone)]
pub struct SnapshotSubRegion {
    base_address: u64,
    size_in_bytes: u64,
}

impl SnapshotSubRegion {
    pub fn new(parent_region: &SnapshotRegion) -> Self {
        Self::new_with_offset_and_size_in_bytes(parent_region.get_base_address(), parent_region.get_region_size())
    }

    pub fn new_with_offset_and_size_in_bytes(base_address: u64, size_in_bytes: u64) -> Self {
        Self {
            base_address,
            size_in_bytes,
        }
    }
    
    /*
    pub fn get_current_values_pointer(&self) -> *const u8 {
        let parent_region = self.parent_region.read().unwrap();
        let current_values = parent_region.get_current_values();
        unsafe { current_values.as_ptr().add(self.base_address) }
    }

    pub fn get_previous_values_pointer(&self) -> *const u8 {
        let parent_region = self.parent_region.read().unwrap();
        let previous_values = parent_region.get_previous_values();
        unsafe { previous_values.as_ptr().add(self.base_address) }
    } */

    pub fn get_base_address(&self) -> u64 {
        return self.base_address;
    }

    pub fn get_end_address(&self) -> u64 {
        return self.get_base_address() + self.size_in_bytes as u64;
    }
    
    pub fn get_size_in_bytes(&self) -> u64 {
        return self.size_in_bytes;
    }

    pub fn get_byte_count(&self) -> u64 {
        return self.size_in_bytes as u64;
    }

    pub fn get_byte_count_for_data_type_size(&self, data_type_size: u64) -> u64 {
        if data_type_size > self.size_in_bytes {
            return 0;
        } else {
            return (self.size_in_bytes - data_type_size) as u64;
        }
    }

    pub fn is_vector_friendly_size(&self, alignment: MemoryAlignment) -> bool {
        return self.get_byte_count() - self.get_misalignment(alignment as u64) > vectors::get_hardware_vector_size();
    }

    pub fn get_element_count(&self, alignment: MemoryAlignment, data_type_size: u64) -> u64 {
        let alignment = max(alignment as u64, 1);
        let misalignment = self.get_misalignment(alignment);
    
        if misalignment >= self.size_in_bytes as u64 {
            return 0;
        }
    
        let effective_size_in_bytes = self.size_in_bytes as u64 - misalignment;
    
        // Ensure that effective_size_in_bytes is at least the size of the data type
        if effective_size_in_bytes < data_type_size as u64 {
            return 0;
        }
    
        let byte_count = effective_size_in_bytes - data_type_size as u64 + 1;
    
        return byte_count / alignment;
    }

    fn get_misalignment(&self, alignment: u64) -> u64 {
        let base_address = self.get_base_address();
        let aligned_base = (base_address + alignment - 1) / alignment * alignment;

        return aligned_base - base_address;
    }
}
