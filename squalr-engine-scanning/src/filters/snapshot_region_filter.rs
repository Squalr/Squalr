use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_architecture::vectors::vectors;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use squalr_engine_memory::normalized_region::NormalizedRegion;
use std::cmp::max;

#[derive(Debug)]
pub struct SnapshotRegionFilter {
    parent_region_index: u64,
    filter_range: NormalizedRegion
}

impl SnapshotRegionFilter {
    pub fn new(parent_region_index: u64, parent_region: &SnapshotRegion) -> Self {
        Self::new_with_address_and_size_in_bytes(parent_region_index, parent_region.get_base_address(), parent_region.get_region_size())
    }

    pub fn new_with_address_and_size_in_bytes(parent_region_index: u64, base_address: u64, size_in_bytes: u64) -> Self {
        Self {
            parent_region_index: parent_region_index,
            filter_range: NormalizedRegion::new(base_address, size_in_bytes),
        }
    }

    pub fn get_base_address(&self) -> u64 {
        return self.filter_range.get_base_address();
    }

    pub fn get_end_address(&self) -> u64 {
        return self.filter_range.get_end_address();
    }
    
    pub fn get_size_in_bytes(&self) -> u64 {
        return self.filter_range.get_region_size();
    }

    pub fn get_byte_count_for_data_type_size(&self, data_type_size: u64) -> u64 {
        if data_type_size > self.get_size_in_bytes() {
            return 0;
        } else {
            return (self.get_size_in_bytes() - data_type_size) as u64;
        }
    }

    pub fn is_vector_friendly_size(&self, alignment: MemoryAlignment) -> bool {
        return self.get_size_in_bytes() - self.get_misalignment(alignment as u64) > vectors::get_hardware_vector_size();
    }

    pub fn get_element_count(&self, alignment: MemoryAlignment, data_type_size: u64) -> u64 {
        let alignment = max(alignment as u64, 1);
        let misalignment = self.get_misalignment(alignment);
    
        if misalignment >= self.get_size_in_bytes() as u64 {
            return 0;
        }
    
        let effective_size_in_bytes = self.get_size_in_bytes() as u64 - misalignment;
    
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
