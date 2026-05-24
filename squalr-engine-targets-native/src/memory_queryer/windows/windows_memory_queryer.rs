use crate::memory_queryer::memory_protection_enum::MemoryProtectionEnum;
use crate::memory_queryer::memory_queryer_trait::MemoryQueryerTrait;
use crate::memory_queryer::memory_type_enum::MemoryTypeEnum;
use crate::memory_queryer::region_bounds_handling::RegionBoundsHandling;
use core::ffi::c_void;
use core::mem::size_of;
use squalr_engine_api::structures::memory::bitness::Bitness;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use std::ffi::OsStr;
use std::path::Path;
use windows_sys::Win32::Foundation::HMODULE;
use windows_sys::Win32::System::Memory::{
    MEMORY_BASIC_INFORMATION64, PAGE_EXECUTE, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_WRITECOPY, PAGE_READWRITE, PAGE_WRITECOPY, VirtualQueryEx,
};
use windows_sys::Win32::System::ProcessStatus::{K32EnumProcessModulesEx, K32GetModuleFileNameExA, K32GetModuleInformation, LIST_MODULES_ALL, MODULEINFO};

pub struct WindowsMemoryQueryer;

impl WindowsMemoryQueryer {
    pub fn new() -> Self {
        WindowsMemoryQueryer
    }

    fn get_protection_flags(
        &self,
        protection: &MemoryProtectionEnum,
    ) -> u32 {
        let mut flags = 0;

        if protection.contains(MemoryProtectionEnum::WRITE) {
            flags |= PAGE_READWRITE | PAGE_EXECUTE_READWRITE;
        }

        if protection.contains(MemoryProtectionEnum::EXECUTE) {
            flags |= PAGE_EXECUTE | PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY;
        }

        if protection.contains(MemoryProtectionEnum::COPY_ON_WRITE) {
            flags |= PAGE_WRITECOPY | PAGE_EXECUTE_WRITECOPY;
        }

        flags
    }
}

impl MemoryQueryerTrait for WindowsMemoryQueryer {
    fn get_virtual_pages(
        &self,
        process_info: &OpenedProcessInfo,
        required_protection: MemoryProtectionEnum,
        excluded_protection: MemoryProtectionEnum,
        allowed_types: MemoryTypeEnum,
        start_address: u64,
        end_address: u64,
        region_bounds_handling: RegionBoundsHandling,
    ) -> Vec<NormalizedRegion> {
        let required_flags = self.get_protection_flags(&required_protection);
        let excluded_flags = self.get_protection_flags(&excluded_protection);
        let mut regions = vec![];
        let mut current_region = NormalizedRegion::new(start_address, 0);

        // Return empty regions if the start or end address is invalid.
        if start_address >= end_address {
            return regions;
        }

        // If partial matches are supported, we need to enumerate all memory regions.
        // A small optimization may be possible here if we start from the max(0, startAddress - max page size) instead.
        if region_bounds_handling == RegionBoundsHandling::Include || region_bounds_handling == RegionBoundsHandling::Resize {
            current_region.set_base_address(0);
        }

        // Start off as a single point at our starting location.
        current_region.set_end_address(current_region.get_base_address());

        // Loop over the entire virtual memory space, advancing by chunks of either allocated or unallocated memory pages.
        while current_region.get_end_address() < end_address {
            // Each iteration, move the base address to the end of the region being searched.
            // For the first iteration, this does nothing. For all other iterations, this advances our search window.
            current_region.set_base_address(current_region.get_end_address());

            // At the current address, query the page of memory at this location.
            let mut mbi: MEMORY_BASIC_INFORMATION64 = unsafe { std::mem::zeroed() };
            let result = unsafe {
                VirtualQueryEx(
                    process_info.get_handle() as *mut c_void,
                    current_region.get_base_address() as *const c_void,
                    &mut mbi as *mut _ as *mut _,
                    size_of::<MEMORY_BASIC_INFORMATION64>(),
                )
            };

            if result == 0 || mbi.RegionSize <= 0 {
                break;
            }

            debug_assert!(mbi.BaseAddress == current_region.get_base_address());
            current_region.set_region_size(mbi.RegionSize as u64);

            // Ensure that the memory is not free (unallocated), not reserved (ie no physical memory assigned yet), and that the memory is committed (allocated and usable).
            if mbi.State == windows_sys::Win32::System::Memory::MEM_FREE
                || mbi.State == windows_sys::Win32::System::Memory::MEM_RESERVE
                || mbi.State != windows_sys::Win32::System::Memory::MEM_COMMIT
            {
                continue;
            }

            // Enforce allowed types.
            if mbi.Type == 0 && !allowed_types.contains(MemoryTypeEnum::NONE) {
                continue;
            } else if mbi.Type == windows_sys::Win32::System::Memory::MEM_PRIVATE && !allowed_types.contains(MemoryTypeEnum::PRIVATE) {
                continue;
            } else if mbi.Type == windows_sys::Win32::System::Memory::MEM_IMAGE && !allowed_types.contains(MemoryTypeEnum::IMAGE) {
                continue;
            } else if mbi.Type == windows_sys::Win32::System::Memory::MEM_MAPPED && !allowed_types.contains(MemoryTypeEnum::MAPPED) {
                continue;
            }

            // Ensure at least one required protection flag is set.
            if required_flags != 0 && (mbi.Protect & required_flags) == 0 {
                continue;
            }

            // Ensure no ignored protection flags are set.
            if excluded_flags != 0 && (mbi.Protect & excluded_flags) != 0 {
                continue;
            }

            // Handle regions that are partially in the provided bounds based on given bounds handling method.
            if current_region.get_base_address() < start_address || current_region.get_end_address() > end_address {
                match region_bounds_handling {
                    RegionBoundsHandling::Exclude => continue,
                    RegionBoundsHandling::Include => {}
                    RegionBoundsHandling::Resize => {
                        current_region.set_end_address(end_address.min(current_region.get_end_address()));
                        current_region.set_base_address_retain_end_address(start_address.max(current_region.get_base_address()));
                    }
                }
            }

            // Return the memory page.
            regions.push(NormalizedRegion::new(mbi.BaseAddress as u64, mbi.RegionSize as u64));
        }

        regions
    }

    fn get_all_virtual_pages(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Vec<NormalizedRegion> {
        let start_address = 0;
        let end_address = self.get_maximum_address(process_info);
        self.get_virtual_pages(
            process_info,
            MemoryProtectionEnum::NONE,
            MemoryProtectionEnum::NONE,
            MemoryTypeEnum::PRIVATE | MemoryTypeEnum::IMAGE | MemoryTypeEnum::MAPPED,
            start_address,
            end_address,
            RegionBoundsHandling::Exclude,
        )
    }

    fn is_address_writable(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
    ) -> bool {
        let start_address = address;
        let end_address = address;
        let virtual_pages_in_bounds = self.get_virtual_pages(
            process_info,
            MemoryProtectionEnum::WRITE,
            MemoryProtectionEnum::NONE,
            MemoryTypeEnum::PRIVATE | MemoryTypeEnum::IMAGE | MemoryTypeEnum::MAPPED,
            start_address,
            end_address,
            RegionBoundsHandling::Include,
        );

        // Check for writability by searching for a page that includes the target address that is writable.
        virtual_pages_in_bounds.len() > 0
    }

    fn get_maximum_address(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> u64 {
        if process_info.get_bitness() == Bitness::Bit32 {
            u32::MAX as u64
        } else {
            u64::MAX
        }
    }

    fn get_min_usermode_address(
        &self,
        _: &OpenedProcessInfo,
    ) -> u64 {
        // In windows, anything below this is not addressable by a normal program.
        0x10000
    }

    fn get_max_usermode_address(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> u64 {
        if process_info.get_bitness() == Bitness::Bit32 {
            // For 32-bit applications, the usermode memory is generally the first 2GB of process RAM.
            // JIRA: Large Address Aware support? This is incredibly rare, but would be more correct to support.
            0x7FFF_FFFF
        } else {
            // In windows, the max usermode address is arbitrarily set to this value for x64.
            0x7FFF_FFFF_FFFF
        }
    }

    fn get_modules(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Vec<NormalizedModule> {
        let mut modules = vec![];
        let mut module_handles: [HMODULE; 1024] = [0 as HMODULE; 1024];
        let mut cb_needed = 0;

        let result = unsafe {
            K32EnumProcessModulesEx(
                process_info.get_handle() as *mut c_void,
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

        for index in 0..num_modules as usize {
            let mut module_path_bytes = vec![0u8; 1024];
            let result = unsafe {
                K32GetModuleFileNameExA(
                    process_info.get_handle() as *mut c_void,
                    module_handles[index],
                    module_path_bytes.as_mut_ptr(),
                    module_path_bytes.len() as u32,
                )
            };

            if result == 0 {
                continue;
            }

            let module_path = String::from_utf8_lossy(&module_path_bytes)
                .trim_end_matches('\0')
                .to_string();
            let module_name = Path::new(&module_path)
                .file_name()
                .unwrap_or_else(|| OsStr::new(""))
                .to_str()
                .unwrap_or("")
                .to_string();
            let mut module_info: MODULEINFO = unsafe { std::mem::zeroed() };

            let result = unsafe {
                K32GetModuleInformation(
                    process_info.get_handle() as *mut c_void,
                    module_handles[index],
                    &mut module_info,
                    std::mem::size_of::<MODULEINFO>() as u32,
                )
            };

            if result == 0 {
                continue;
            }

            modules.push(NormalizedModule::new(
                &module_name,
                module_info.lpBaseOfDll as u64,
                module_info.SizeOfImage as u64,
            ));
        }

        modules
    }

    fn address_to_module(
        &self,
        address: u64,
        modules: &Vec<NormalizedModule>,
    ) -> Option<(String, u64)> {
        for module in modules {
            if module.contains_address(address) {
                return Some((module.get_module_name().to_string(), address - module.get_base_address()));
            }
        }

        None
    }

    fn resolve_module(
        &self,
        modules: &Vec<NormalizedModule>,
        identifier: &str,
    ) -> u64 {
        if !identifier.is_empty() {
            for module in modules {
                if module
                    .get_module_name()
                    .trim()
                    .eq_ignore_ascii_case(identifier.trim())
                {
                    return module.get_base_address();
                }
            }
        }

        0
    }
}
