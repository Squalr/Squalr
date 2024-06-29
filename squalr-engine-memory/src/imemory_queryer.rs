use std::collections::HashSet;
use sysinfo::Pid;
use crate::emulator_type::EmulatorType;
use crate::normalized_module::NormalizedModule;
use crate::normalized_region::NormalizedRegion;
use crate::memory_protection_enum::MemoryProtectionEnum;
use crate::memory_type_enum::MemoryTypeEnum;
use crate::region_bounds_handling::RegionBoundsHandling;

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
        emulator_type: EmulatorType,
    ) -> HashSet<NormalizedRegion>;

    fn get_all_virtual_pages(
        &self,
        process_id: &Pid,
        emulator_type: EmulatorType,
    ) -> HashSet<NormalizedRegion>;

    fn is_address_writable(&self, process_id: &Pid, address: u64) -> bool;

    fn get_maximum_address(&self, process_id: &Pid) -> u64;

    fn get_min_usermode_address(&self, process_id: &Pid) -> u64;

    fn get_max_usermode_address(&self, process_id: &Pid) -> u64;

    fn get_modules(&self, process_id: &Pid, emulator_type: EmulatorType) -> HashSet<NormalizedModule>;

    fn get_stack_addresses(&self, process_id: &Pid, emulator_type: EmulatorType) -> HashSet<NormalizedRegion>;

    fn get_heap_addresses(&self, process_id: &Pid, emulator_type: EmulatorType) -> HashSet<NormalizedRegion>;

    fn address_to_module(&self, process_id: &Pid, address: u64, module_name: &mut String, emulator_type: EmulatorType) -> u64;

    fn resolve_module(&self, process_id: &Pid, identifier: &str, emulator_type: EmulatorType) -> u64;
}
