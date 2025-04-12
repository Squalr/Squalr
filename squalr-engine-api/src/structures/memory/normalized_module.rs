use crate::structures::memory::normalized_region::NormalizedRegion;
use std::hash::{Hash, Hasher};

pub struct NormalizedModule {
    base_region: NormalizedRegion,
    module_name: String,
}

impl NormalizedModule {
    pub fn new(
        module_name: &str,
        base_address: u64,
        size: u64,
    ) -> Self {
        Self {
            base_region: NormalizedRegion::new(base_address, size),
            module_name: module_name.to_string(),
        }
    }

    pub fn new_from_normalized_region(
        normalized_region: NormalizedRegion,
        module_name: &str,
    ) -> Self {
        Self {
            base_region: normalized_region,
            module_name: module_name.to_string(),
        }
    }

    pub fn into_base_region(self) -> NormalizedRegion {
        self.base_region
    }

    pub fn get_module_name(&self) -> &str {
        &self.module_name
    }

    pub fn get_base_address(&self) -> u64 {
        self.base_region.get_base_address()
    }

    pub fn set_base_address(
        &mut self,
        base_address: u64,
    ) {
        self.base_region.set_base_address(base_address);
    }

    pub fn get_region_size(&self) -> u64 {
        self.base_region.get_region_size()
    }

    pub fn set_region_size(
        &mut self,
        region_size: u64,
    ) {
        self.base_region.set_region_size(region_size);
    }

    pub fn contains_address(
        &self,
        address: u64,
    ) -> bool {
        self.base_region.contains_address(address)
    }

    pub fn get_base_region(&self) -> &NormalizedRegion {
        return &self.base_region;
    }
}

impl PartialEq for NormalizedModule {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.base_region == other.base_region && self.module_name == other.module_name
    }
}

impl Eq for NormalizedModule {}

impl Hash for NormalizedModule {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        self.base_region.hash(state);
        self.module_name.hash(state);
    }
}
