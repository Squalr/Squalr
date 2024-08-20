use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::constraints::scan_constraint::ScanConstraint;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use squalr_engine_common::dynamic_struct::data_type::{self, DataType};
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use squalr_engine_memory::memory_reader::memory_reader_trait::IMemoryReader;
use squalr_engine_memory::memory_reader::MemoryReader;
use squalr_engine_memory::normalized_region::NormalizedRegion;
use std::borrow::BorrowMut;
use std::collections::HashMap;

#[derive(Debug)]
pub struct SnapshotRegion {
    normalized_region: NormalizedRegion,
    current_values: Vec<u8>,
    previous_values: Vec<u8>,
    filters: HashMap<DataType, Vec<SnapshotRegionFilter>>,
}

impl SnapshotRegion {
    pub fn new(
        normalized_region: NormalizedRegion,
    ) -> Self {
        Self {
            normalized_region: normalized_region,
            current_values: vec![],
            previous_values: vec![],
            filters: HashMap::new(),
        }
    }

    pub fn get_current_values(&self) -> &Vec<u8> {
        return &self.current_values;
    }

    pub fn get_previous_values(&self) -> &Vec<u8> {
        return &self.previous_values;
    }

    pub fn read_all_memory(&mut self, process_handle: u64) -> Result<(), String> {
        let region_size = self.get_region_size() as usize;
    
        std::mem::swap(&mut self.current_values, &mut self.previous_values);
        
        if self.current_values.is_empty() && region_size > 0 {
            self.current_values = vec![0u8; region_size];
        }
    
        let result = MemoryReader::get_instance().read_bytes(process_handle, self.get_base_address(), &mut self.current_values)?;
    
        return Ok(result);
    }

    pub fn read_all_memory_parallel(&mut self, process_handle: u64) -> Result<(), String> {
        let chunk_size = 2 << 23; // 16MB seems to be the optimal value for my CPU
        let region_size = self.get_region_size() as usize;

        if region_size <= chunk_size {
            return self.read_all_memory(process_handle);
        }

        std::mem::swap(&mut self.current_values, &mut self.previous_values);
    
        if self.current_values.is_empty() && region_size > 0 {
            self.current_values = vec![0u8; region_size];
        }
    
        // Split the memory region into chunks and process them in parallel
        let base_address = self.get_base_address();
        let mut chunks: Vec<_> = self.current_values.chunks_mut(chunk_size).collect();
    
        chunks
            .par_iter_mut()
            .enumerate()
            .try_for_each(|(index, chunk)| {
                let offset = index * chunk_size;
                MemoryReader::get_instance().read_bytes(process_handle, base_address + offset as u64, chunk)
            })
            .map_err(|e| e.to_string())?;
    
        return Ok(());
    }
    
    pub fn get_current_values_pointer(&self, snapshot_region_filter: &SnapshotRegionFilter) -> *const u8 {
        unsafe{
            let offset = snapshot_region_filter.get_base_address() - self.get_base_address();
            return self.get_current_values().as_ptr().add(offset as usize);
        }
    }
    
    pub fn get_previous_values_pointer(&self, snapshot_region_filter: &SnapshotRegionFilter) -> *const u8 {
        unsafe{
            let offset = snapshot_region_filter.get_base_address() - self.get_base_address();
            return self.get_previous_values().as_ptr().add(offset as usize);
        }
    }
    
    pub fn get_base_address(&self) -> u64 {
        return self.normalized_region.get_base_address();
    }
    
    pub fn get_end_address(&self) -> u64 {
        return self.normalized_region.get_base_address() + self.normalized_region.get_region_size();
    }

    pub fn get_region_size(&self) -> u64 {
        return self.normalized_region.get_region_size();
    }

    pub fn set_alignment(&mut self, alignment: MemoryAlignment) {
        self.normalized_region.set_alignment(alignment);
    }

    pub fn has_current_values(&self) -> bool {
        return !self.current_values.is_empty();
    }

    pub fn has_previous_values(&self) -> bool {
        return !self.previous_values.is_empty();
    }

    pub fn can_compare_with_constraint(&self, constraints: &ScanConstraint) -> bool {
        if !constraints.is_valid() || !self.has_current_values() {
            return false;
        }

        if !constraints.is_immediate_constraint() && !self.has_previous_values() {
            return false;
        }

        return true;
    }

    pub fn create_filters(&mut self, data_types: &Vec<DataType>) {
        // Collect the data types that need new filters
        let new_filters: Vec<DataType> = data_types.iter()
            .filter(|data_type| !self.filters.contains_key(data_type))
            .cloned()
            .collect();

        // Insert new filters outside of the iteration loop
        for data_type in new_filters {
            self.filters.insert(
                data_type.clone(),
                vec![SnapshotRegionFilter::new(
                    self.get_base_address(),
                    self.get_region_size(),
                )],
            );
        }
    }

    pub fn get_filters(&self) -> &HashMap<DataType, Vec<SnapshotRegionFilter>> {
        // Retrieve references for all specified filters
        return &self.filters;
    }
}
