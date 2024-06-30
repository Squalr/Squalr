use squalr_engine_memory::normalized_region::NormalizedRegion;

#[derive(Debug, Clone)]
pub struct SnapshotRegion {
    normalized_region: NormalizedRegion,
    pub current_values: Vec<u8>,
    pub previous_values: Vec<u8>,
}

impl SnapshotRegion {
    pub fn new(base_address: u64, current_values: Vec<u8>, region_size: u64) -> Self {
        Self {
            normalized_region: NormalizedRegion::new(base_address, region_size),
            current_values,
            previous_values: Vec::new(),
        }
    }

    pub fn new_from_normalized_region(normalized_region: NormalizedRegion) -> Self {
        let region_size = normalized_region.get_region_size();
        let current_values = vec![0; region_size as usize]; // Initialize with zeroes or fetch actual values if needed
        Self {
            normalized_region,
            current_values,
            previous_values: Vec::new(),
        }
    }

    pub fn set_current_values(&mut self, values: Vec<u8>) {
        self.previous_values = std::mem::replace(&mut self.current_values, values);
    }

    pub fn get_base_address(&self) -> u64 {
        self.normalized_region.get_base_address()
    }

    pub fn get_region_size(&self) -> u64 {
        self.normalized_region.get_region_size()
    }

    // Implement other methods by delegating to `normalized_region`
    // For example:
    pub fn align(&mut self, alignment: u32) {
        self.normalized_region.align(alignment);
    }
}