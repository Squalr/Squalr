use crate::scanners::constraints::scan_constraint::ScanConstraint;
use crate::snapshots::snapshot_sub_region::SnapshotSubRegion;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use squalr_engine_memory::memory_reader::MemoryReader;
use squalr_engine_memory::memory_reader::memory_reader_trait::IMemoryReader;
use squalr_engine_memory::normalized_region::NormalizedRegion;

#[derive(Debug)]
pub struct SnapshotRegion {
    normalized_region: NormalizedRegion,
    current_values: Vec<u8>,
    previous_values: Vec<u8>,
    snapshot_sub_regions: Vec<SnapshotSubRegion>,
}

impl SnapshotRegion {
    pub fn new(base_address: u64, region_size: u64) -> Self {
        Self {
            normalized_region: NormalizedRegion::new(base_address, region_size),
            current_values: Vec::new(),
            previous_values: Vec::new(),
            snapshot_sub_regions: Vec::new(),
        }
    }

    pub fn new_from_normalized_region(normalized_region: NormalizedRegion) -> Self {
        Self {
            normalized_region,
            current_values: Vec::new(),
            previous_values: Vec::new(),
            snapshot_sub_regions: Vec::new(),
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
            .try_for_each(|(i, chunk)| {
                let offset = i * chunk_size;
                MemoryReader::get_instance().read_bytes(process_handle, base_address + offset as u64, chunk)
            })
            .map_err(|e| e.to_string())?;
    
        return Ok(());
    }
    
    pub fn get_sub_region_current_values_pointer(&self, snapshot_sub_region: &SnapshotSubRegion) -> *const u8 {
        let current_values = self.get_current_values();
        unsafe { current_values.as_ptr().add((snapshot_sub_region.get_base_address() - self.get_base_address()) as usize) }
    }
    
    pub fn get_sub_region_previous_values_pointer(&self, snapshot_sub_region: &SnapshotSubRegion) -> *const u8 {
        let current_values = self.get_current_values();
        unsafe { current_values.as_ptr().add((snapshot_sub_region.get_base_address() - self.get_base_address()) as usize) }
    }
    
    pub fn get_base_address(&self) -> u64 {
        return self.normalized_region.get_base_address();
    }

    pub fn get_region_size(&self) -> u64 {
        return self.normalized_region.get_region_size();
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
        if !constraints.is_valid() || !self.has_current_values() || (constraints.is_relative_constraint() && !self.has_previous_values()) {
            return false;
        }

        return true;
    }
}
