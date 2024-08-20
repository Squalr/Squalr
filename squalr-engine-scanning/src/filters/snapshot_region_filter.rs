use crate::scanners::constraints::scan_constraint::ScanConstraint;
use squalr_engine_architecture::vectors::vectors;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use squalr_engine_memory::normalized_region::NormalizedRegion;
use std::cmp::max;

#[derive(Debug)]
pub struct SnapshotRegionFilter {
    filter_range: NormalizedRegion
}

impl SnapshotRegionFilter {
    pub fn new(base_address: u64, size_in_bytes: u64) -> Self {
        Self {
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

    /*
    pub fn can_compare_with_constraint(&self, constraints: &ScanConstraint) -> bool {
        if !constraints.is_valid() || !self.parent_snapshot.read().unwrap().has_current_values() {
            return false;
        }

        if !constraints.is_immediate_constraint() && !self.has_previous_values() {
            return false;
        }

        return true;
    }

    

    pub fn get_byte_count(&self) -> u64 {
        return self.snapshot_sub_regions.iter().map(|sub_region| sub_region.get_byte_count()).sum();
    }

    pub fn get_element_count(&self, alignment: MemoryAlignment, data_type_size: u64) -> u64 {
        return self.snapshot_sub_regions.iter().map(|sub_region| sub_region.get_element_count(alignment, data_type_size)).sum();
    }
    
    pub fn set_snapshot_sub_regions(&mut self, snapshot_sub_regions: Vec<SnapshotSubRegion>) {
        self.snapshot_sub_regions = snapshot_sub_regions;
    }

    pub fn get_snapshot_sub_regions(&self) -> &Vec<SnapshotSubRegion> {
        return &self.snapshot_sub_regions;
    }
    
    pub fn get_snapshot_sub_regions_create_if_none(&mut self) -> Vec<SnapshotSubRegion> {
        if self.snapshot_sub_regions.is_empty() && self.get_region_size() > 0 {
            self.snapshot_sub_regions.push(SnapshotSubRegion::new(self));
        }

        return self.snapshot_sub_regions.clone();
    }

     */
}
