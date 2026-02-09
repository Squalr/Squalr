use crate::memory_queryer::memory_protection_enum::MemoryProtectionEnum;
use crate::memory_queryer::memory_type_enum::MemoryTypeEnum;
use crate::memory_queryer::region_bounds_handling::RegionBoundsHandling;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;

pub trait MemoryQueryerTrait {
    fn get_virtual_pages(
        &self,
        process_info: &OpenedProcessInfo,
        required_protection: MemoryProtectionEnum,
        excluded_protection: MemoryProtectionEnum,
        allowed_types: MemoryTypeEnum,
        start_address: u64,
        end_address: u64,
        region_bounds_handling: RegionBoundsHandling,
    ) -> Vec<NormalizedRegion>;

    fn get_all_virtual_pages(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Vec<NormalizedRegion>;

    fn is_address_writable(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
    ) -> bool;

    fn get_maximum_address(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> u64;

    fn get_min_usermode_address(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> u64;

    fn get_max_usermode_address(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> u64;

    fn get_modules(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Vec<NormalizedModule>;

    fn address_to_module(
        &self,
        address: u64,
        modules: &Vec<NormalizedModule>,
    ) -> Option<(String, u64)>;

    fn resolve_module(
        &self,
        modules: &Vec<NormalizedModule>,
        identifier: &str,
    ) -> u64;
}
