use crate::memory_queryer::memory_protection_enum::MemoryProtectionEnum;
use crate::memory_queryer::memory_queryer_trait::MemoryQueryerTrait;
use crate::memory_queryer::memory_type_enum::MemoryTypeEnum;
use crate::memory_queryer::region_bounds_handling::RegionBoundsHandling;
use libc::{PROC_PIDPATHINFO_MAXSIZE, c_int, c_void, proc_regionfilename};
use mach2::kern_return::KERN_SUCCESS;
use mach2::message::mach_msg_type_number_t;
use mach2::vm::mach_vm_region_recurse;
use mach2::vm_prot::{VM_PROT_COPY, VM_PROT_EXECUTE, VM_PROT_READ, VM_PROT_WRITE};
use mach2::vm_region::{
    SM_COW, SM_PRIVATE, SM_PRIVATE_ALIASED, SM_SHARED, SM_SHARED_ALIASED, SM_TRUESHARED, vm_region_recurse_info_t, vm_region_submap_info_64,
};
use mach2::vm_types::{mach_vm_address_t, mach_vm_size_t, natural_t};
use squalr_engine_api::structures::memory::bitness::Bitness;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use std::collections::HashMap;
use std::path::Path;

pub struct MacOsMemoryQueryer;

struct MacOsRegionInfo {
    base_address: u64,
    region_size: u64,
    protection_flags: u32,
    share_mode: u8,
    mapped_file_path: Option<String>,
}

impl MacOsMemoryQueryer {
    pub fn new() -> Self {
        MacOsMemoryQueryer
    }

    fn query_region_file_path(
        process_id: u32,
        address: u64,
    ) -> Option<String> {
        let mut path_bytes = vec![0u8; PROC_PIDPATHINFO_MAXSIZE as usize];
        let path_length = unsafe { proc_regionfilename(process_id as c_int, address, path_bytes.as_mut_ptr() as *mut c_void, path_bytes.len() as u32) };

        if path_length <= 0 {
            return None;
        }

        let path_length = path_length as usize;
        let path_string = String::from_utf8_lossy(&path_bytes[..path_length])
            .trim_end_matches('\0')
            .to_string();

        if path_string.is_empty() { None } else { Some(path_string) }
    }

    fn query_regions(
        process_info: &OpenedProcessInfo,
        start_address: u64,
        end_address: u64,
    ) -> Vec<MacOsRegionInfo> {
        let process_handle = process_info.get_handle();
        if process_handle == 0 || start_address >= end_address {
            return Vec::new();
        }

        let mut region_infos = Vec::new();
        let mut query_address = start_address as mach_vm_address_t;
        let mut query_depth: natural_t = 0;

        while query_address < end_address as mach_vm_address_t {
            let mut region_base_address = query_address;
            let mut region_size: mach_vm_size_t = 0;
            let mut region_info = vm_region_submap_info_64::default();
            let mut region_info_count: mach_msg_type_number_t = vm_region_submap_info_64::count();

            let query_status = unsafe {
                mach_vm_region_recurse(
                    process_handle as _,
                    &mut region_base_address as *mut mach_vm_address_t,
                    &mut region_size as *mut mach_vm_size_t,
                    &mut query_depth as *mut natural_t,
                    &mut region_info as *mut vm_region_submap_info_64 as vm_region_recurse_info_t,
                    &mut region_info_count as *mut mach_msg_type_number_t,
                )
            };

            if query_status != KERN_SUCCESS || region_size == 0 {
                break;
            }

            if region_info.is_submap != 0 {
                query_depth = query_depth.saturating_add(1);
                query_address = region_base_address;
                continue;
            }

            let next_query_address = region_base_address.saturating_add(region_size);
            if next_query_address <= query_address {
                break;
            }

            query_address = next_query_address;

            if region_base_address >= end_address as mach_vm_address_t {
                break;
            }

            region_infos.push(MacOsRegionInfo {
                base_address: region_base_address,
                region_size,
                protection_flags: region_info.protection as u32,
                share_mode: region_info.share_mode,
                mapped_file_path: Self::query_region_file_path(process_info.get_process_id_raw(), region_base_address),
            });
        }

        region_infos
    }

    fn clamp_region_to_bounds(
        region_info: &MacOsRegionInfo,
        start_address: u64,
        end_address: u64,
        region_bounds_handling: RegionBoundsHandling,
    ) -> Option<NormalizedRegion> {
        let region_start_address = region_info.base_address;
        let region_end_address = region_start_address.saturating_add(region_info.region_size);

        if region_end_address <= start_address || region_start_address >= end_address {
            return None;
        }

        match region_bounds_handling {
            RegionBoundsHandling::Exclude => {
                if region_start_address >= start_address && region_end_address <= end_address {
                    Some(NormalizedRegion::new(
                        region_start_address,
                        region_end_address.saturating_sub(region_start_address),
                    ))
                } else {
                    None
                }
            }
            RegionBoundsHandling::Include => Some(NormalizedRegion::new(region_start_address, region_info.region_size)),
            RegionBoundsHandling::Resize => {
                let clamped_start_address = region_start_address.max(start_address);
                let clamped_end_address = region_end_address.min(end_address);

                if clamped_end_address > clamped_start_address {
                    Some(NormalizedRegion::new(
                        clamped_start_address,
                        clamped_end_address.saturating_sub(clamped_start_address),
                    ))
                } else {
                    None
                }
            }
        }
    }

    fn get_protection_flags(
        &self,
        protection: &MemoryProtectionEnum,
    ) -> u32 {
        let mut protection_flags = 0;

        if protection.contains(MemoryProtectionEnum::READ) {
            protection_flags |= VM_PROT_READ as u32;
        }

        if protection.contains(MemoryProtectionEnum::WRITE) {
            protection_flags |= VM_PROT_WRITE as u32;
        }

        if protection.contains(MemoryProtectionEnum::EXECUTE) {
            protection_flags |= VM_PROT_EXECUTE as u32;
        }

        if protection.contains(MemoryProtectionEnum::COPY_ON_WRITE) {
            protection_flags |= VM_PROT_COPY as u32;
        }

        protection_flags
    }

    fn get_memory_type_flags(region_info: &MacOsRegionInfo) -> MemoryTypeEnum {
        let mut memory_type_flags = MemoryTypeEnum::empty();

        if region_info.mapped_file_path.is_none() {
            memory_type_flags |= MemoryTypeEnum::NONE;
        }

        match region_info.share_mode {
            SM_COW | SM_PRIVATE | SM_PRIVATE_ALIASED => memory_type_flags |= MemoryTypeEnum::PRIVATE,
            SM_SHARED | SM_TRUESHARED | SM_SHARED_ALIASED => memory_type_flags |= MemoryTypeEnum::MAPPED,
            _ => {}
        }

        if region_info.mapped_file_path.is_some() && (region_info.protection_flags & VM_PROT_EXECUTE as u32) != 0 {
            memory_type_flags |= MemoryTypeEnum::IMAGE;
        }

        if memory_type_flags.is_empty() {
            memory_type_flags |= MemoryTypeEnum::NONE;
        }

        memory_type_flags
    }

    fn module_name_from_path(module_path: &str) -> String {
        Path::new(module_path)
            .file_name()
            .and_then(|module_name| module_name.to_str())
            .filter(|module_name| !module_name.is_empty())
            .unwrap_or(module_path)
            .to_string()
    }
}

impl MemoryQueryerTrait for MacOsMemoryQueryer {
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
        if start_address >= end_address {
            return Vec::new();
        }

        let query_start_address = if region_bounds_handling == RegionBoundsHandling::Include || region_bounds_handling == RegionBoundsHandling::Resize {
            0
        } else {
            start_address
        };

        let required_protection_flags = self.get_protection_flags(&required_protection);
        let excluded_protection_flags = self.get_protection_flags(&excluded_protection);
        let allowed_type_flags = allowed_types.bits();
        let queried_regions = Self::query_regions(process_info, query_start_address, end_address);

        queried_regions
            .iter()
            .filter_map(|region_info| {
                if required_protection_flags != 0 && (region_info.protection_flags & required_protection_flags) == 0 {
                    return None;
                }

                if excluded_protection_flags != 0 && (region_info.protection_flags & excluded_protection_flags) != 0 {
                    return None;
                }

                let memory_type_flags = Self::get_memory_type_flags(region_info);
                if allowed_type_flags != 0 && (memory_type_flags.bits() & allowed_type_flags) == 0 {
                    return None;
                }

                Self::clamp_region_to_bounds(region_info, start_address, end_address, region_bounds_handling)
            })
            .collect()
    }

    fn get_all_virtual_pages(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Vec<NormalizedRegion> {
        self.get_virtual_pages(
            process_info,
            MemoryProtectionEnum::NONE,
            MemoryProtectionEnum::NONE,
            MemoryTypeEnum::NONE | MemoryTypeEnum::PRIVATE | MemoryTypeEnum::IMAGE | MemoryTypeEnum::MAPPED,
            0,
            self.get_maximum_address(process_info),
            RegionBoundsHandling::Exclude,
        )
    }

    fn is_address_writable(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
    ) -> bool {
        let regions_at_address = Self::query_regions(process_info, address, address.saturating_add(1));

        regions_at_address.iter().any(|region_info| {
            let region_end_address = region_info.base_address.saturating_add(region_info.region_size);
            address >= region_info.base_address && address < region_end_address && (region_info.protection_flags & VM_PROT_WRITE as u32) != 0
        })
    }

    fn get_maximum_address(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> u64 {
        if process_info.get_bitness() == Bitness::Bit32 {
            u32::MAX as u64
        } else {
            0x0000_7FFF_FFFF_FFFF
        }
    }

    fn get_min_usermode_address(
        &self,
        _: &OpenedProcessInfo,
    ) -> u64 {
        0
    }

    fn get_max_usermode_address(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> u64 {
        self.get_maximum_address(process_info)
    }

    fn get_modules(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Vec<NormalizedModule> {
        let mut module_ranges: HashMap<String, (u64, u64)> = HashMap::new();
        let queried_regions = Self::query_regions(process_info, 0, self.get_maximum_address(process_info));

        for region_info in queried_regions {
            if (region_info.protection_flags & VM_PROT_EXECUTE as u32) == 0 {
                continue;
            }

            let mapped_file_path = match region_info.mapped_file_path {
                Some(mapped_file_path) => mapped_file_path,
                None => continue,
            };

            let region_start_address = region_info.base_address;
            let region_end_address = region_start_address.saturating_add(region_info.region_size);
            let module_range_entry = module_ranges
                .entry(mapped_file_path)
                .or_insert((region_start_address, region_end_address));

            module_range_entry.0 = module_range_entry.0.min(region_start_address);
            module_range_entry.1 = module_range_entry.1.max(region_end_address);
        }

        let mut modules: Vec<NormalizedModule> = module_ranges
            .iter()
            .filter_map(|(module_path, (module_start_address, module_end_address))| {
                let module_region_size = module_end_address.saturating_sub(*module_start_address);
                if module_region_size == 0 {
                    return None;
                }

                Some(NormalizedModule::new(
                    &Self::module_name_from_path(module_path),
                    *module_start_address,
                    module_region_size,
                ))
            })
            .collect();

        modules.sort_by_key(|module| module.get_base_address());
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
        let normalized_identifier = identifier.trim();
        if normalized_identifier.is_empty() {
            return 0;
        }

        for module in modules {
            if module
                .get_module_name()
                .trim()
                .eq_ignore_ascii_case(normalized_identifier)
            {
                return module.get_base_address();
            }
        }

        0
    }
}
