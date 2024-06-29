use std::cmp::{Ord, Ordering};
use std::ops::Add;

#[derive(Debug, Clone)]
pub struct NormalizedRegion {
    base_address: u64,
    region_size: i32,
}

impl NormalizedRegion {
    pub fn new() -> Self {
        Self {
            base_address: 0,
            region_size: 0,
        }
    }

    pub fn with_base_and_size(base_address: u64, region_size: i32) -> Self {
        let mut region = Self::new();
        region.generic_constructor(base_address, region_size);
        region
    }

    pub fn get_base_address(&self) -> u64 {
        self.base_address
    }

    pub fn set_base_address(&mut self, base_address: u64) {
        self.base_address = base_address;
    }

    pub fn get_region_size(&self) -> i32 {
        self.region_size
    }

    pub fn set_region_size(&mut self, region_size: i32) {
        self.region_size = region_size;
    }

    pub fn get_end_address(&self) -> u64 {
        self.base_address.add(self.region_size as u64)
    }

    pub fn set_end_address(&mut self, end_address: u64) {
        self.region_size = (end_address - self.base_address) as i32;
    }

    pub fn generic_constructor(&mut self, base_address: u64, region_size: i32) {
        self.base_address = base_address;
        self.region_size = region_size;
    }

    pub fn align(&mut self, alignment: u32) {
        let alignment_value = alignment as i32;

        if alignment_value <= 0 || self.base_address % alignment as u64 == 0 {
            return;
        }

        let end_address = self.get_end_address();
        self.base_address -= self.base_address % alignment as u64;
        self.base_address += alignment as u64;
        self.set_end_address(end_address);
    }

    pub fn contains_address(&self, address: u64) -> bool {
        address >= self.base_address && address <= self.get_end_address()
    }

    pub fn expand(&mut self, expand_size: i32) {
        self.base_address -= expand_size as u64;
        self.region_size += expand_size * 2;
    }

    pub fn chunk_normalized_region(&self, chunk_size: i32) -> Vec<NormalizedRegion> {
        if chunk_size <= 0 {
            eprintln!("Invalid chunk size specified for region");
            return Vec::new();
        }

        let chunk_size = chunk_size.min(self.region_size);
        let chunk_count = (self.region_size / chunk_size) + (if self.region_size % chunk_size == 0 { 0 } else { 1 });
        let mut chunks = Vec::with_capacity(chunk_count as usize);

        for index in 0..chunk_count {
            let mut size = chunk_size;

            if index == chunk_count - 1 && self.region_size > chunk_size && self.region_size % chunk_size != 0 {
                size = self.region_size % chunk_size;
            }

            chunks.push(NormalizedRegion::with_base_and_size(
                self.base_address + (chunk_size as u64 * index as u64),
                size,
            ));
        }

        chunks
    }
}

impl PartialEq for NormalizedRegion {
    fn eq(&self, other: &Self) -> bool {
        self.base_address == other.base_address
    }
}

impl Eq for NormalizedRegion {}

impl PartialOrd for NormalizedRegion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NormalizedRegion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.base_address.cmp(&other.base_address)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MemoryAlignment {
    Auto = 0,
    Alignment1 = 1,
    Alignment2 = 2,
    Alignment4 = 4,
    Alignment8 = 8,
}
