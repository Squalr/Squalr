use crate::imemory_queryer::IMemoryQueryer;
use crate::normalized_region::NormalizedRegion;
use crate::normalized_module::NormalizedModule;
use crate::memory_protection_enum::MemoryProtectionEnum;
use crate::memory_type_enum::MemoryTypeEnum;
use crate::region_bounds_handling::RegionBoundsHandling;
use crate::emulator_type::EmulatorType;
use std::collections::HashSet;
use std::ptr::null_mut;
use sysinfo::Pid;
use winapi::shared::minwindef::HMODULE;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::psapi::{EnumProcessModulesEx, GetModuleFileNameExA, GetModuleInformation, MODULEINFO, LIST_MODULES_ALL};
use winapi::um::memoryapi::VirtualQueryEx;
use winapi::um::winnt::{HANDLE, PROCESS_QUERY_INFORMATION, MEMORY_BASIC_INFORMATION, PROCESS_VM_READ};

pub struct WindowsMemoryQuery;

impl WindowsMemoryQuery {
    pub fn new() -> Self {
        Self
    }

    fn open_process(&self, process_id:  &Pid) -> HANDLE {
        unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, process_id.as_u32()) }
    }
}

impl IMemoryQueryer for WindowsMemoryQuery {
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
    ) -> HashSet<NormalizedRegion> {
        // Implement actual functionality here
        let process_handle = self.open_process(process_id);
        let mut regions = HashSet::new();

        // Define protection and type flags
        // TODO: Translate `required_protection` and `excluded_protection` to actual Windows protection flags
        // TODO: Translate `allowed_types` to actual Windows memory type flags

        let mut address = start_address;
        let mut mbi: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };

        while address < end_address {
            let result = unsafe {
                VirtualQueryEx(
                    process_handle,
                    address as *const _,
                    &mut mbi,
                    std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                )
            };

            if result == 0 {
                break;
            }

            // Check protection and type
            // TODO: Add actual checks for protection and type

            // Add region to the set
            // regions.insert(NormalizedRegion::new(address, mbi.RegionSize as usize));

            address += mbi.RegionSize as u64;
        }

        return regions;
    }

    fn get_all_virtual_pages(
        &self,
        process_id: &Pid,
        emulator_type: EmulatorType,
    ) -> HashSet<NormalizedRegion> {
        let start_address = 0;
        let end_address = self.get_maximum_address(process_id);
        self.get_virtual_pages(
            process_id,
            MemoryProtectionEnum::empty(),
            MemoryProtectionEnum::empty(),
            MemoryTypeEnum::all(),
            start_address,
            end_address,
            RegionBoundsHandling::Exclude,
            emulator_type,
        )
    }

    fn is_address_writable(&self, process_id: &Pid, address: u64) -> bool {
        // Implement actual functionality here
        let process_handle = self.open_process(process_id);
        let mut mbi: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };

        let result = unsafe {
            VirtualQueryEx(
                process_handle,
                address as *const _,
                &mut mbi,
                std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
            )
        };

        if result == 0 {
            return false;
        }

        // Check if the memory is writable
        // TODO: Add actual check for writable protection flags
        return false;
    }

    fn get_maximum_address(&self, process_id: &Pid) -> u64 {
        // Implement actual functionality here
        // TODO: Determine the maximum address based on the system architecture (x86 or x64)
        u64::MAX
    }

    fn get_min_usermode_address(&self, process_id: &Pid) -> u64 {
        // Implement actual functionality here
        // TODO: Determine the minimum user mode address
        0x10000 // Example value for Windows
    }

    fn get_max_usermode_address(&self, process_id: &Pid) -> u64 {
        // Implement actual functionality here
        // TODO: Determine the maximum user mode address based on the system architecture (x86 or x64)
        0x7FFFFFFF_FFFF // Example value for 64-bit Windows
    }

    fn get_modules(
        &self,
        process_id: &Pid,
        emulator_type: EmulatorType,
    ) -> HashSet<NormalizedModule> {
        // Implement actual functionality here
        let process_handle = self.open_process(process_id);
        let mut modules = HashSet::new();

        let mut module_handles: [HMODULE; 1024] = [null_mut(); 1024];
        let mut cb_needed = 0;

        let result = unsafe {
            EnumProcessModulesEx(
                process_handle,
                module_handles.as_mut_ptr(),
                std::mem::size_of_val(&module_handles) as u32,
                &mut cb_needed,
                LIST_MODULES_ALL,
            )
        };

        if result == 0 {
            return modules;
        }

        let num_modules = cb_needed / std::mem::size_of::<HMODULE>() as u32;

        for i in 0..num_modules as usize {
            let mut module_name = vec![0u8; 1024];
            let result = unsafe {
                GetModuleFileNameExA(
                    process_handle,
                    module_handles[i],
                    module_name.as_mut_ptr() as *mut i8,
                    module_name.len() as u32,
                )
            };

            if result == 0 {
                continue;
            }

            let module_name = String::from_utf8_lossy(&module_name).to_string();
            let mut module_info: MODULEINFO = unsafe { std::mem::zeroed() };

            let result = unsafe {
                GetModuleInformation(
                    process_handle,
                    module_handles[i],
                    &mut module_info,
                    std::mem::size_of::<MODULEINFO>() as u32,
                )
            };

            if result == 0 {
                continue;
            }
            
            /*
            modules.insert(NormalizedModule::new(
                module_name,
                module_info.lpBaseOfDll as u64,
                module_info.SizeOfImage as usize,
            )); */
        }

        return modules;
    }

    fn get_stack_addresses(
        &self,
        process_id: &Pid,
        emulator_type: EmulatorType,
    ) -> HashSet<NormalizedRegion> {
        // Implement actual functionality here
        unimplemented!()
    }

    fn get_heap_addresses(
        &self,
        process_id: &Pid,
        emulator_type: EmulatorType,
    ) -> HashSet<NormalizedRegion> {
        // Implement actual functionality here
        unimplemented!()
    }

    fn address_to_module(
        &self,
        process_id: &Pid,
        address: u64,
        module_name: &mut String,
        emulator_type: EmulatorType,
    ) -> u64 {
        let modules = self.get_modules(process_id, emulator_type);
        
        /*
        for module in modules {
            if module.contains_address(address) {
                *module_name = module.get_name();
                return address - module.get_base_address();
            }
        }

        *module_name = String::new(); */
        return address;
    }

    fn resolve_module(
        &self,
        process_id: &Pid,
        identifier: &str,
        emulator_type: EmulatorType,
    ) -> u64 {
        let modules = self.get_modules(process_id, emulator_type);

        for module in modules {
            if module.get_name().eq_ignore_ascii_case(identifier) {
                return module.get_base_address();
            }
        }

        return 0;
    }
}
