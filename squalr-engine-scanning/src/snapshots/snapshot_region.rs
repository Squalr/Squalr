use squalr_engine_memory::memory_alignment::MemoryAlignment;
use squalr_engine_memory::normalized_region::NormalizedRegion;

#[derive(Debug)]
pub struct SnapshotRegion {
    normalized_region: NormalizedRegion,
    values: Vec<u8>,
}

impl SnapshotRegion {
    pub fn new(
        normalized_region: NormalizedRegion,
        values: Vec<u8>,
    ) -> Self {
        Self {
            normalized_region: normalized_region,
            values: values,
        }
    }

    pub fn get_values(&self) -> &Vec<u8> {
        return &self.values;
    }

    /*
    pub fn read_all_memory(&mut self, process_handle: u64) -> Result<(), String> {
        let region_size = self.get_region_size() as usize;
    
        std::mem::swap(&mut self.values, &mut self.previous_values);
        
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
    } */
    
    pub fn get_values_pointer(&self) -> *const u8 {
        return self.get_values().as_ptr();
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

    pub fn has_values(&self) -> bool {
        return !self.values.is_empty();
    }
}
