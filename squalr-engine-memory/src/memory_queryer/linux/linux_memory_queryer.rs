use crate::imemory_queryer::IMemoryQueryer;
use crate::normalized_region::NormalizedRegion;
use crate::normalized_module::NormalizedModule;
use crate::memory_protection_enum::MemoryProtectionEnum;
use crate::memory_type_enum::MemoryTypeEnum;
use crate::region_bounds_handling::RegionBoundsHandling;
use std::collections::HashSet;
use std::process_id::Pid;

pub struct LinuxMemoryQuery;

impl LinuxMemoryQuery {
    pub fn new() -> Self {
        Self
    }
}

impl IMemoryQueryer for LinuxMemoryQuery {
    fn get_virtual_pages(
        &self,
        process_id: &Pid,
        required_protection: MemoryProtectionEnum,
        excluded_protection: MemoryProtectionEnum,
        allowed_types: MemoryTypeEnum,
        start_address: u64,
        end_address: u64,
        region_bounds_handling: RegionBoundsHandling,
    ) -> HashSet<NormalizedRegion> {
        // Implementation here
        unimplemented!()
    }

    fn get_all_virtual_pages(
        &self,
        process_id: &Pid,
    ) -> HashSet<NormalizedRegion> {
        // Implementation here
        unimplemented!()
    }

    fn is_address_writable(&self, process_id: &Pid, address: u64) -> bool {
        // Implementation here
        unimplemented!()
    }

    fn get_maximum_address(&self, process_id: &Pid) -> u64 {
        // Implementation here
        unimplemented!()
    }

    fn get_min_usermode_address(&self, process_id: &Pid) -> u64 {
        // Implementation here
        unimplemented!()
    }

    fn get_max_usermode_address(&self, process_id: &Pid) -> u64 {
        // Implementation here
        unimplemented!()
    }

    fn get_modules(&self, process_id: &Pid) -> HashSet<NormalizedModule> {
        // Implementation here
        unimplemented!()
    }

    fn get_stack_addresses(&self, process_id: &Pid) -> HashSet<NormalizedRegion> {
        // Implementation here
        unimplemented!()
    }

    fn get_heap_addresses(&self, process_id: &Pid) -> HashSet<NormalizedRegion> {
        // Implementation here
        unimplemented!()
    }

    fn address_to_module(&self, process_id: &Pid, address: u64, module_name: &mut String) -> u64 {
        // Implementation here
        unimplemented!()
    }

    fn resolve_module(&self, process_id: &Pid, identifier: &str) -> u64 {
        // Implementation here
        unimplemented!()
    }
}
