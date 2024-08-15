use crate::scanners::constraints::scan_constraint::ScanConstraint;
use crate::snapshots::snapshot_element_range::SnapshotElementRange;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use squalr_engine_memory::memory_reader::MemoryReader;
use squalr_engine_memory::memory_reader::memory_reader_trait::IMemoryReader;
use squalr_engine_memory::normalized_region::NormalizedRegion;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct SnapshotRegion {
    normalized_region: NormalizedRegion,
    current_values: Arc<RwLock<Vec<u8>>>,
    previous_values: Arc<RwLock<Vec<u8>>>,
    snapshot_element_ranges: Vec<Arc<RwLock<SnapshotElementRange>>>,
    element_count : u32,
}

impl SnapshotRegion {
    pub fn new(base_address: u64, region_size: u64) -> Self {
        Self {
            normalized_region: NormalizedRegion::new(base_address, region_size),
            current_values: Arc::new(RwLock::new(Vec::new())),
            previous_values: Arc::new(RwLock::new(Vec::new())),
            snapshot_element_ranges: Vec::new(),
            element_count: 0,
        }
    }

    pub fn new_from_normalized_region(normalized_region: NormalizedRegion) -> Self {
        Self {
            normalized_region,
            current_values: Arc::new(RwLock::new(Vec::new())),
            previous_values: Arc::new(RwLock::new(Vec::new())),
            snapshot_element_ranges: Vec::new(),
            element_count: 0,
        }
    }

    pub fn set_current_values(&mut self, values: Vec<u8>) {
        let new_values = Arc::new(RwLock::new(values));
        self.previous_values = std::mem::replace(&mut self.current_values, new_values);
    }

    pub fn get_current_values(&self) -> &Arc<RwLock<Vec<u8>>> {
        return &self.current_values;
    }

    pub fn get_previous_values(&self) -> &Arc<RwLock<Vec<u8>>> {
        return &self.previous_values;
    }

    pub fn read_all_memory(&mut self, process_handle: u64) -> Result<(), String> {
        let region_size = self.get_region_size();
        let mut new_values = vec![0u8; region_size as usize];
        
        let result = MemoryReader::get_instance().read_bytes(process_handle, self.get_base_address(), &mut new_values)?;

        self.set_current_values(new_values);

        return Ok(result);
    }

    pub fn get_base_address(&self) -> u64 {
        return self.normalized_region.get_base_address();
    }

    pub fn get_region_size(&self) -> u64 {
        return self.normalized_region.get_region_size();
    }

    pub fn get_element_count(&self, alignment: MemoryAlignment, data_type_size: usize) -> u64 {
        return self.snapshot_element_ranges.clone().into_iter().map(|range| range.as_ref().read().unwrap().get_aligned_element_count(alignment)).sum();
    }

    pub fn set_snapshot_element_ranges(&mut self, snapshot_element_ranges: Vec<Arc<RwLock<SnapshotElementRange>>>) {
        self.snapshot_element_ranges = snapshot_element_ranges;
    }
    
    pub fn get_snapshot_element_ranges(&self) -> Vec<Arc<RwLock<SnapshotElementRange>>> {
        self.snapshot_element_ranges.clone()
    }    

    pub fn set_byte_alignment(&mut self, alignment: MemoryAlignment) {
        self.normalized_region.set_byte_alignment(alignment);
    }

    pub fn set_data_type_size(&mut self, data_type_size: usize) {
        self.element_count = 0;
        panic!("todo");
    }

    pub fn has_current_values(&self) -> bool {
        return !self.current_values.as_ref().read().unwrap().is_empty();
    }

    pub fn has_previous_values(&self) -> bool {
        return !self.previous_values.as_ref().read().unwrap().is_empty();
    }

    pub fn can_compare_with_constraint(&self, constraints: &ScanConstraint) -> bool {
        if !constraints.is_valid() || !self.has_current_values() || (constraints.is_relative_constraint() && !self.has_previous_values()) {
            return false;
        }

        return true;
    }
}
