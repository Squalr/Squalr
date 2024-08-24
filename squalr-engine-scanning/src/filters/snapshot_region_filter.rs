use squalr_engine_architecture::vectors::vectors;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use squalr_engine_memory::normalized_region::NormalizedRegion;
use std::cmp::max;

/// Defines a range of filtered memory within a snapshot region. These filters are created by
/// scans to narrow down on the desired addresses.
#[derive(Debug)]
pub struct SnapshotRegionFilter {
    filter_range: NormalizedRegion,
}

impl SnapshotRegionFilter {
    pub fn new(
        base_address: u64,
        size_in_bytes: u64
    ) -> Self {
        Self {
            filter_range: NormalizedRegion::new(base_address, size_in_bytes),
        }
    }

    pub fn get_base_address(
        &self,
    ) -> u64 {
        return self.filter_range.get_base_address();
    }

    pub fn set_base_address(
        &mut self,
        base_address: u64,
    ) {
        self.filter_range.set_base_address(base_address);
    }

    pub fn set_end_address(
        &mut self,
        end_address: u64,
    ) {
        self.filter_range.set_end_address(end_address);
    }

    pub fn get_end_address(
        &self,
    ) -> u64 {
        return self.filter_range.get_end_address();
    }
    
    pub fn get_region_size(
        &self,
    ) -> u64 {
        return self.filter_range.get_region_size();
    }

    pub fn is_vector_friendly_size(
        &self,
        alignment: MemoryAlignment
    ) -> bool {
        return self.get_region_size() - self.get_misalignment(alignment as u64) > vectors::get_hardware_vector_size();
    }

    pub fn get_element_count(
        &self,
        alignment: MemoryAlignment,
        data_type_size: u64
    ) -> u64 {
        let alignment = max(alignment as u64, 1);
        let misalignment = self.get_misalignment(alignment);
    
        if misalignment >= self.get_region_size() as u64 {
            return 0;
        }
    
        let effective_size_in_bytes = self.get_region_size() as u64 - misalignment;
    
        // Ensure that effective_size_in_bytes is at least the size of the data type
        if effective_size_in_bytes < data_type_size as u64 {
            return 0;
        }
    
        let byte_count = effective_size_in_bytes;
    
        return byte_count / alignment;
    }

    fn get_misalignment(
        &self,
        alignment: u64
    ) -> u64 {
        let base_address = self.get_base_address();
        let aligned_base = (base_address + alignment - 1) / alignment * alignment;

        return aligned_base - base_address;
    }
}
