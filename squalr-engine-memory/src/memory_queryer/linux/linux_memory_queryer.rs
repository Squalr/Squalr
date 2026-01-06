use crate::memory_queryer::memory_protection_enum::MemoryProtectionEnum;
use crate::memory_queryer::memory_queryer_trait::IMemoryQueryer;
use crate::memory_queryer::memory_type_enum::MemoryTypeEnum;
use crate::memory_queryer::region_bounds_handling::RegionBoundsHandling;

use squalr_engine_api::structures::memory::bitness::Bitness;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;

use std::cell::OnceCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::{LazyLock, RwLock};

pub struct LinuxMemoryQueryer;

static MODULE_CACHE: LazyLock<RwLock<HashMap<u32, Vec<NormalizedModule>>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

#[derive(Debug)]
#[allow(dead_code)]
struct ProcPidMapsEntry {
    // man 5 /proc/pid/maps
    address_start: u64,
    address_end: u64,
    page_permissions: MemoryProtectionEnum,
    file_offset: u64,
    device: String,
    inode: String,
    path_name: String,
}

impl LinuxMemoryQueryer {
    pub fn new() -> Self {
        LinuxMemoryQueryer
    }

    fn get_proc_maps(pid: u32) -> Vec<ProcPidMapsEntry> {
        let mut parsed_maps = vec![];

        let maps_path = format!("/proc/{}/maps", pid);
        let maps_file = match File::open(maps_path) {
            Ok(f) => f,
            Err(_) => return parsed_maps,
        };

        let reader = BufReader::new(maps_file);
        for line_result in reader.lines() {
            let line = match line_result {
                Ok(l) => l,
                Err(_) => continue,
            };

            parsed_maps.push(Self::parse_map_entry(line.as_str()));
        }

        parsed_maps
    }

    fn parse_map_entry(entry: &str) -> ProcPidMapsEntry {
        let mut split = entry.splitn(6, ' ');

        let address_range = split.next().unwrap();
        let mut address_split = address_range.splitn(2, '-');
        let address_start = u64::from_str_radix(address_split.next().unwrap(), 16).expect("Failed to parse address_start");
        let address_end = u64::from_str_radix(address_split.next().unwrap(), 16).expect("Failed to parse address_end");
        let page_permissions = Self::parse_page_permissions(split.next().expect("Failed to parse permissions"));
        let file_offset = u64::from_str_radix(split.next().unwrap(), 16).expect("Failed to parse file offset");
        let device = split.next().unwrap().to_string();
        let inode = split.next().unwrap().to_string();
        let path_name = split.next().unwrap().to_string();

        ProcPidMapsEntry {
            address_start,
            address_end,
            page_permissions,
            file_offset,
            device,
            inode,
            path_name,
        }
    }

    fn parse_page_permissions(pp: &str) -> MemoryProtectionEnum {
        let from_char = |ch: u8| match ch {
            b'r' => MemoryProtectionEnum::READ,
            b'w' => MemoryProtectionEnum::WRITE,
            b'x' => MemoryProtectionEnum::EXECUTE,
            b's' => MemoryProtectionEnum::SHARED,
            b'p' => MemoryProtectionEnum::COPY_ON_WRITE,
            _ => MemoryProtectionEnum::NONE,
        };

        pp.bytes()
            .map(from_char)
            .fold(MemoryProtectionEnum::NONE, std::ops::BitOr::bitor)
    }
}

impl IMemoryQueryer for LinuxMemoryQueryer {
    fn get_virtual_pages(
        &self,
        process_info: &OpenedProcessInfo,
        required_protection: MemoryProtectionEnum,
        excluded_protection: MemoryProtectionEnum,
        _allowed_types: MemoryTypeEnum,
        start_address: u64,
        end_address: u64,
        region_bounds_handling: RegionBoundsHandling,
    ) -> Vec<NormalizedRegion> {
        Self::get_proc_maps(process_info.get_process_id_raw())
            .into_iter()
            .filter_map(|map_entry| {
                if !map_entry.page_permissions.contains(required_protection) {
                    return None;
                }

                if map_entry.page_permissions.intersects(excluded_protection) {
                    return None;
                }

                if map_entry.address_start < start_address || map_entry.address_end > end_address {
                    match region_bounds_handling {
                        RegionBoundsHandling::Exclude => return None,
                        // TODO: Handle Resize
                        _ => {}
                    }
                }

                // TODO: allowed_types... unsure if this is relevant on Linux

                Some(NormalizedRegion::new(map_entry.address_start, map_entry.address_end - map_entry.address_start))
            })
            .collect()
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
            MemoryTypeEnum::NONE,
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
        let virtual_pages_in_bounds = self.get_virtual_pages(
            process_info,
            MemoryProtectionEnum::WRITE,
            MemoryProtectionEnum::NONE,
            MemoryTypeEnum::PRIVATE | MemoryTypeEnum::IMAGE | MemoryTypeEnum::MAPPED,
            address,
            address,
            RegionBoundsHandling::Include,
        );

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
        // TODO: Check this.
        0x10000
    }

    fn get_max_usermode_address(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> u64 {
        // TODO: Check this.
        if process_info.get_bitness() == Bitness::Bit32 {
            0x7FFF_FFFF
        } else {
            0x7FFF_FFFF_FFFF
        }
    }

    fn get_modules(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Vec<NormalizedModule> {
        if let Ok(mut module_cache) = MODULE_CACHE.write() {
            // We don't have the luxury of EnumProcessModules on Linux, instead:
            // 1. Index /proc/pid/maps entries by path_name
            // 2. Filter out irrelevant entries (ie. uninitialized/stack/shared/heap regions)
            // 3. Look at the first and last entries for each module to calculate the region size
            //    and construct a NormalizedModule
            // 4. Since this is somewhat expensive to do, cache the results

            let pid = process_info.get_process_id_raw();
            let maps_by_pathname: OnceCell<HashMap<String, Vec<ProcPidMapsEntry>>> = OnceCell::new();

            return module_cache
                .entry(pid)
                .or_insert(
                    maps_by_pathname
                        .get_or_init(|| {
                            let mut map: HashMap<String, Vec<ProcPidMapsEntry>> = HashMap::new();
                            for entry in Self::get_proc_maps(pid) {
                                map.entry(entry.path_name.clone())
                                    .or_insert(Vec::new())
                                    .push(entry);
                            }
                            map
                        })
                        .into_iter()
                        .filter_map(|(path_name, entries)| {
                            if path_name.is_empty() || path_name.starts_with("[") {
                                return None;
                            }

                            let first_address = entries.first().unwrap().address_start;
                            let last_address = entries.last().unwrap().address_end;
                            let region_size = last_address - first_address;

                            Some(NormalizedModule::new(path_name.as_str(), first_address, region_size))
                        })
                        .collect(),
                )
                .to_vec();
        }

        vec![]
    }

    fn address_to_module(
        &self,
        address: u64,
        modules: &Vec<NormalizedModule>,
    ) -> Option<(String, u64)> {
        modules
            .into_iter()
            .find(|it| it.contains_address(address))
            .map(|it| (it.get_module_name().to_string(), address - it.get_base_address()))
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
