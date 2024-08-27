use crate::memory_alignment::MemoryAlignment;
use std::cmp::{Ord, Ordering};
use std::hash::{Hash, Hasher};
use std::ops::Add;

/// Defines a generic range of addresses, with no extra information.
/// This is the base type for many more specialized regions.
#[derive(Debug)]
pub struct NormalizedRegion {
    base_address: u64,
    region_size: u64,
}

impl NormalizedRegion {
    pub fn new(
        base_address: u64,
        region_size: u64,
    ) -> Self {
        Self {
            base_address: base_address,
            region_size: region_size,
        }
    }

    pub fn get_base_address(&self) -> u64 {
        return self.base_address;
    }

    pub fn set_base_address(
        &mut self,
        base_address: u64,
    ) {
        self.base_address = base_address;
    }

    pub fn get_region_size(&self) -> u64 {
        return self.region_size;
    }

    pub fn set_region_size(
        &mut self,
        region_size: u64,
    ) {
        self.region_size = region_size;
    }

    pub fn get_end_address(&self) -> u64 {
        return self.base_address.add(self.region_size as u64);
    }

    pub fn set_end_address(
        &mut self,
        end_address: u64,
    ) {
        self.region_size = (end_address - self.base_address) as u64;
    }

    pub fn generic_constructor(
        &mut self,
        base_address: u64,
        region_size: u64,
    ) {
        self.base_address = base_address;
        self.region_size = region_size;
    }

    pub fn set_alignment(
        &mut self,
        alignment: MemoryAlignment,
    ) {
        let alignment_value = alignment as u64;

        if alignment_value <= 0 || self.base_address % alignment as u64 == 0 {
            return;
        }

        let end_address = self.get_end_address();
        self.base_address -= self.base_address % alignment as u64;
        self.base_address += alignment as u64;
        self.set_end_address(end_address);
    }

    pub fn contains_address(
        &self,
        address: u64,
    ) -> bool {
        return address >= self.base_address && address <= self.get_end_address();
    }

    pub fn expand(
        &mut self,
        expand_size: u64,
    ) {
        self.base_address -= expand_size as u64;
        self.region_size += expand_size * 2;
    }
}

impl PartialEq for NormalizedRegion {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        return self.base_address == other.base_address;
    }
}

impl Eq for NormalizedRegion {}

impl PartialOrd for NormalizedRegion {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        return Some(self.cmp(other));
    }
}

impl Ord for NormalizedRegion {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        return self.base_address.cmp(&other.base_address);
    }
}

impl Hash for NormalizedRegion {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        self.base_address.hash(state);
        self.region_size.hash(state);
    }
}
