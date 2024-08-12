use squalr_engine_memory::memory_reader::MemoryReader;
use squalr_engine_memory::memory_reader::memory_reader_trait::IMemoryReader;
use squalr_engine_memory::normalized_region::NormalizedRegion;

#[derive(Debug)]
pub struct SnapshotRegion {
    normalized_region: NormalizedRegion,
    pub current_values: Vec<u8>,
    pub previous_values: Vec<u8>,
}

impl SnapshotRegion {
    pub fn new(base_address: u64, region_size: u64) -> Self {
        Self {
            normalized_region: NormalizedRegion::new(base_address, region_size),
            current_values: Vec::new(),
            previous_values: Vec::new(),
        }
    }

    pub fn new_from_normalized_region(normalized_region: NormalizedRegion) -> Self {
        Self {
            normalized_region,
            current_values: Vec::new(),
            previous_values: Vec::new(),
        }
    }

    pub fn set_current_values(&mut self, values: Vec<u8>) {
        self.previous_values = std::mem::replace(&mut self.current_values, values);
    }

    pub fn read_all_memory(&mut self, process_handle: u64) -> Result<(), String> {
        let region_size = self.get_region_size();
        self.current_values.resize(region_size as usize, 0);
        MemoryReader::instance().read_bytes(process_handle, self.get_base_address(), &mut self.current_values)
    }

    pub fn get_base_address(&self) -> u64 {
        self.normalized_region.get_base_address()
    }

    pub fn get_region_size(&self) -> u64 {
        self.normalized_region.get_region_size()
    }

    pub fn align(&mut self, alignment: u32) {
        self.normalized_region.align(alignment);
    }
}
