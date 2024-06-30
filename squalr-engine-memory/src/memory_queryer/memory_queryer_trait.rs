use sysinfo::Pid;
use crate::memory_queryer::memory_protection_enum::MemoryProtectionEnum;
use crate::memory_queryer::memory_type_enum::MemoryTypeEnum;
use crate::memory_queryer::region_bounds_handling::RegionBoundsHandling;
use crate::normalized_module::NormalizedModule;
use crate::normalized_region::NormalizedRegion;

pub trait IMemoryQueryer {
    fn get_virtual_pages(
        &self,
        process_id: &Pid,
        required_protection: MemoryProtectionEnum,
        excluded_protection: MemoryProtectionEnum,
        allowed_types: MemoryTypeEnum,
        start_address: u64,
        end_address: u64,
        region_bounds_handling: RegionBoundsHandling,
    ) -> Vec<NormalizedRegion>;

    fn get_all_virtual_pages(
        &self,
        process_id: &Pid,
    ) -> Vec<NormalizedRegion>;

    fn is_address_writable(&self, process_id: &Pid, address: u64) -> bool;

    fn get_maximum_address(&self, process_id: &Pid) -> u64;

    fn get_min_usermode_address(&self, process_id: &Pid) -> u64;

    fn get_max_usermode_address(&self, process_id: &Pid) -> u64;

    fn get_modules(&self, process_id: &Pid) -> Vec<NormalizedModule>;

    fn address_to_module(&self, process_id: &Pid, address: u64, module_name: &mut String) -> u64;

    fn resolve_module(&self, process_id: &Pid, identifier: &str) -> u64;
}
