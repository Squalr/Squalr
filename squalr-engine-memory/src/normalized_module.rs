use std::path::Path;
use std::ffi::OsStr;
use super::normalized_region::NormalizedRegion;

pub struct NormalizedModule {
    base_region: NormalizedRegion,
    name: String,
    full_path: String,
}

impl NormalizedModule {
    pub fn new(full_path: &str, base_address: u64, size: i32) -> Self {
        let name = Path::new(full_path)
            .file_name()
            .unwrap_or_else(|| OsStr::new(""))
            .to_str()
            .unwrap_or("")
            .to_string();

        Self {
            base_region: NormalizedRegion::with_base_and_size(base_address, size),
            name,
            full_path: full_path.to_string(),
        }
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

    pub fn set_base_address(&mut self, base_address: u64) {
        self.base_region.set_base_address(base_address);
    }

    pub fn get_region_size(&self) -> i32 {
        self.base_region.get_region_size()
    }

    pub fn set_region_size(&mut self, region_size: i32) {
        self.base_region.set_region_size(region_size);
    }
}

