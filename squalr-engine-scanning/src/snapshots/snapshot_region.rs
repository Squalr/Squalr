use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use squalr_engine_memory::memory_reader::memory_reader_trait::IMemoryReader;
use squalr_engine_memory::memory_reader::MemoryReader;
use squalr_engine_memory::normalized_region::NormalizedRegion;
use squalr_engine_common::values::data_type::DataType;
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

    pub fn get_current_values(
        &self
    ) -> &Vec<u8> {
        return &self.current_values;
    }

    pub fn get_previous_values(
        &self
    ) -> &Vec<u8> {
        return &self.previous_values;
    }

    pub fn read_all_memory(
        &mut self,
        process_handle: u64
    ) -> Result<(), String> {
        let region_size = self.get_region_size() as usize;
    
        std::mem::swap(&mut self.current_values, &mut self.previous_values);
        
        if self.current_values.is_empty() && region_size > 0 {
            self.current_values = vec![0u8; region_size];
        }
    
        let result = MemoryReader::get_instance().read_bytes(process_handle, self.get_base_address(), &mut self.current_values)?;
    
        return Ok(result);
    }

    pub fn read_all_memory_parallel(
        &mut self,
        process_handle: u64
    ) -> Result<(), String> {
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
    
    pub fn get_current_values_pointer(
        &self,
        snapshot_region_filter: &SnapshotRegionFilter
    ) -> *const u8 {
        unsafe{
            let offset = snapshot_region_filter.get_base_address() - self.get_base_address();
            return self.get_current_values().as_ptr().add(offset as usize);
        }
    }
    
    pub fn get_previous_values_pointer(
        &self,
        snapshot_region_filter: &SnapshotRegionFilter
    ) -> *const u8 {
        unsafe{
            let offset = snapshot_region_filter.get_base_address() - self.get_base_address();
            return self.get_previous_values().as_ptr().add(offset as usize);
        }
    }
    
    pub fn get_base_address(
        &self
    ) -> u64 {
        return self.normalized_region.get_base_address();
    }
    
    pub fn get_end_address(
        &self
    ) -> u64 {
        return self.normalized_region.get_base_address() + self.normalized_region.get_region_size();
    }

    pub fn get_region_size(
        &self
    ) -> u64 {
        return self.normalized_region.get_region_size();
    }

    pub fn set_alignment(
        &mut self,
        alignment: MemoryAlignment
    ) {
        self.normalized_region.set_alignment(alignment);
    }

    pub fn has_current_values(
        &self
    ) -> bool {
        return !self.current_values.is_empty();
    }

    pub fn has_previous_values(
        &self
    ) -> bool {
        return !self.previous_values.is_empty();
    }

    pub fn can_compare_using_parameters(
        &self,
        scan_parameters: &ScanParameters
    ) -> bool {
        if !scan_parameters.is_valid() || !self.has_current_values() {
            return false;
        }

        if !scan_parameters.is_immediate_comparison() && !self.has_previous_values() {
            return false;
        }

        return true;
    }

    /// Creates the initial set of filters for the given set of scan filter parameters.
    /// At first, these filters are equal in size to the entire snapshot region.
    pub fn create_initial_scan_results(
        &mut self,
        scan_filter_parameters: &Vec<ScanFilterParameters>
    ) {
        let base_address = self.get_base_address();
        let region_size = self.get_region_size();
    
        for scan_filter_parameter in scan_filter_parameters {
            self.filters.entry(scan_filter_parameter.get_data_type().clone()).or_insert_with(|| {
                vec![SnapshotRegionFilter::new(base_address, region_size)]
            });
        }
    }

    /// Updates the set of filters over this snapshot region. Filters are essentially scan results for a given data type.
    /// Additionally, we resize this region to reduce wasted memory (ie data outside the min/max filter addresses).
    pub fn set_all_filters(
        &mut self,
        filters: HashMap<DataType, Vec<SnapshotRegionFilter>>,
    ) {
        self.filters = filters;
    
        if self.filters.is_empty() {
            self.normalized_region.set_region_size(0);
            return;
        }
    
        let mut new_base_address = u64::MAX;
        let mut new_end_address = 0u64;
        let mut found_valid_filter = false;
    
        for filter_vec in self.filters.values() {
            for filter in filter_vec {
                let filter_base = filter.get_base_address();
                let filter_end = filter.get_end_address();
    
                if filter_base < new_base_address {
                    new_base_address = filter_base;
                }
                if filter_end > new_end_address {
                    new_end_address = filter_end;
                }
                found_valid_filter = true;
            }
        }
    
        if !found_valid_filter {
            self.normalized_region.set_region_size(0);
            return;
        }
    
        let original_base_address = self.get_base_address();
        let original_end_address = self.get_end_address();
    
        new_base_address = new_base_address.max(original_base_address);
        new_end_address = new_end_address.min(original_end_address);
    
        let new_region_size = new_end_address - new_base_address;
        self.normalized_region.set_base_address(new_base_address);
        self.normalized_region.set_region_size(new_region_size);
    
        let start_offset = (new_base_address - original_base_address) as usize;
        let new_length = (new_end_address - new_base_address + 1) as usize;
    
        if !self.current_values.is_empty() {
            self.current_values.drain(..start_offset);
            self.current_values.truncate(new_length);
        }
    
        if !self.previous_values.is_empty() {
            self.previous_values.drain(..start_offset);
            self.previous_values.truncate(new_length);
        }
    }

    pub fn get_filters(
        &self
    ) -> &HashMap<DataType, Vec<SnapshotRegionFilter>> {
        return &self.filters;
    }
}
