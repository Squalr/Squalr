use super::normalized_region::NormalizedRegion;
use std::ffi::OsStr;
use std::hash::{
    Hash,
    Hasher,
};
use std::path::Path;

#[derive(Debug)]
pub struct NormalizedModule {
    base_region: NormalizedRegion,
    name: String,
    full_path: String,
}

impl NormalizedModule {
    pub fn new(
        full_path: &str,
        base_address: u64,
        size: u64,
    ) -> Self {
        let name = Path::new(full_path)
            .file_name()
            .unwrap_or_else(|| OsStr::new(""))
            .to_str()
            .unwrap_or("")
            .to_string();

        Self {
            base_region: NormalizedRegion::new(base_address, size),
            name,
            full_path: full_path.to_string(),
        }
    }

    pub fn new_from_normalized_region(
        normalized_region: NormalizedRegion,
        full_path: &str,
    ) -> Self {
        let name = Path::new(full_path)
            .file_name()
            .unwrap_or_else(|| OsStr::new(""))
            .to_str()
            .unwrap_or("")
            .to_string();

        Self {
            base_region: normalized_region,
            name,
            full_path: full_path.to_string(),
        }
    }

    pub fn into_base_region(self) -> NormalizedRegion {
        self.base_region
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_full_path(&self) -> &str {
        &self.full_path
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
        self.base_region == other.base_region && self.name == other.name && self.full_path == other.full_path
    }
}

impl Eq for NormalizedModule {}

impl Hash for NormalizedModule {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        self.base_region.hash(state);
        self.name.hash(state);
        self.full_path.hash(state);
    }
}
