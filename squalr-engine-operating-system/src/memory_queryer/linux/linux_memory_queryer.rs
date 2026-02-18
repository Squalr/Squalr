use crate::memory_queryer::memory_protection_enum::MemoryProtectionEnum;
use crate::memory_queryer::memory_queryer_trait::MemoryQueryerTrait;
use crate::memory_queryer::memory_type_enum::MemoryTypeEnum;
use crate::memory_queryer::region_bounds_handling::RegionBoundsHandling;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;

pub struct LinuxMemoryQueryer;

impl LinuxMemoryQueryer {
    pub fn new() -> Self {
        LinuxMemoryQueryer
    }
}

impl MemoryQueryerTrait for LinuxMemoryQueryer {
    fn get_virtual_pages(
        &self,
        _process_info: &OpenedProcessInfo,
        _required_protection: MemoryProtectionEnum,
        _excluded_protection: MemoryProtectionEnum,
        _allowed_types: MemoryTypeEnum,
        _start_address: u64,
        _end_address: u64,
        _region_bounds_handling: RegionBoundsHandling,
    ) -> Vec<NormalizedRegion> {
        vec![]
    }

    fn get_all_virtual_pages(
        &self,
        _process_info: &OpenedProcessInfo,
    ) -> Vec<NormalizedRegion> {
        vec![]
    }

    fn is_address_writable(
        &self,
        _process_info: &OpenedProcessInfo,
        _address: u64,
    ) -> bool {
        false
    }

    fn get_maximum_address(
        &self,
        _process_info: &OpenedProcessInfo,
    ) -> u64 {
        0
    }

    fn get_min_usermode_address(
        &self,
        _: &OpenedProcessInfo,
    ) -> u64 {
        0
    }

    fn get_max_usermode_address(
        &self,
        _process_info: &OpenedProcessInfo,
    ) -> u64 {
        0
    }

    fn get_modules(
        &self,
        _process_info: &OpenedProcessInfo,
    ) -> Vec<NormalizedModule> {
        vec![]
    }

    fn address_to_module(
        &self,
        _address: u64,
        _modules: &Vec<NormalizedModule>,
    ) -> Option<(String, u64)> {
        None
    }

    fn resolve_module(
        &self,
        _modules: &Vec<NormalizedModule>,
        _identifier: &str,
    ) -> u64 {
        0
    }
}
