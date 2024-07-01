use crate::memory_queryer::memory_protection_enum::MemoryProtectionEnum;
use crate::memory_queryer::memory_type_enum::MemoryTypeEnum;
use crate::memory_queryer::region_bounds_handling::RegionBoundsHandling;
use crate::normalized_module::NormalizedModule;
use crate::normalized_region::NormalizedRegion;

use squalr_engine_processes::process_info::ProcessInfo;

pub trait IMemoryQueryer {
    fn get_virtual_pages(
        &self,
        process_info: &ProcessInfo,
        required_protection: MemoryProtectionEnum,
        excluded_protection: MemoryProtectionEnum,
        allowed_types: MemoryTypeEnum,
        start_address: u64,
        end_address: u64,
        region_bounds_handling: RegionBoundsHandling,
    ) -> Vec<NormalizedRegion>;

    fn get_all_virtual_pages(
        &self,
        process_info: &ProcessInfo,
    ) -> Vec<NormalizedRegion>;

    fn is_address_writable(&self, process_info: &ProcessInfo, address: u64) -> bool;

    fn get_maximum_address(&self, process_info: &ProcessInfo) -> u64;

    fn get_min_usermode_address(&self, process_info: &ProcessInfo) -> u64;

    fn get_max_usermode_address(&self, process_info: &ProcessInfo) -> u64;

    fn get_modules(&self, process_info: &ProcessInfo) -> Vec<NormalizedModule>;

    fn address_to_module(&self, process_info: &ProcessInfo, address: u64, module_name: &mut String) -> u64;

    fn resolve_module(&self, process_info: &ProcessInfo, identifier: &str) -> u64;
}
