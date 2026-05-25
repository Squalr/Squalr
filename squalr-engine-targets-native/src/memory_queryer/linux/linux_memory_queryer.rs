use crate::memory_queryer::memory_protection_enum::MemoryProtectionEnum;
use crate::memory_queryer::memory_queryer_trait::MemoryQueryerTrait;
use crate::memory_queryer::memory_type_enum::MemoryTypeEnum;
use crate::memory_queryer::region_bounds_handling::RegionBoundsHandling;
use squalr_engine_api::structures::memory::bitness::Bitness;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct LinuxMemoryQueryer;

struct ProcMapsRegion {
    start_address: u64,
    end_address: u64,
    permissions: String,
    pathname: String,
}

#[derive(Default)]
struct SmapsRegionInfo {
    rss_bytes: u64,
    vm_flags: HashSet<String>,
}

impl LinuxMemoryQueryer {
    pub fn new() -> Self {
        LinuxMemoryQueryer
    }

    fn parse_proc_maps(process_id: u32) -> std::io::Result<Vec<ProcMapsRegion>> {
        let process_maps_path = format!("/proc/{process_id}/maps");
        let maps_file = File::open(process_maps_path)?;
        let maps_reader = BufReader::new(maps_file);
        let mut parsed_regions = Vec::new();

        for maps_line_result in maps_reader.lines() {
            let maps_line = maps_line_result?;

            if let Some(parsed_region) = Self::parse_maps_line(&maps_line) {
                parsed_regions.push(parsed_region);
            }
        }

        Ok(parsed_regions)
    }

    fn parse_proc_smaps(process_id: u32) -> std::io::Result<HashMap<u64, SmapsRegionInfo>> {
        let process_smaps_path = format!("/proc/{process_id}/smaps");
        let smaps_file = File::open(process_smaps_path)?;
        let smaps_reader = BufReader::new(smaps_file);
        let mut smaps_region_info_by_start_address = HashMap::new();
        let mut current_region_start_address = None;
        let mut current_region_info = SmapsRegionInfo::default();

        for smaps_line_result in smaps_reader.lines() {
            let smaps_line = smaps_line_result?;

            if let Some(parsed_region) = Self::parse_maps_line(&smaps_line) {
                if let Some(region_start_address) = current_region_start_address {
                    smaps_region_info_by_start_address.insert(region_start_address, current_region_info);
                }

                current_region_start_address = Some(parsed_region.start_address);
                current_region_info = SmapsRegionInfo::default();
                continue;
            }

            if let Some(rss_bytes) = Self::parse_smaps_kilobyte_field(&smaps_line, "Rss:") {
                current_region_info.rss_bytes = rss_bytes;
                continue;
            }

            if let Some(vm_flags) = Self::parse_smaps_vm_flags(&smaps_line) {
                current_region_info.vm_flags = vm_flags;
            }
        }

        if let Some(region_start_address) = current_region_start_address {
            smaps_region_info_by_start_address.insert(region_start_address, current_region_info);
        }

        Ok(smaps_region_info_by_start_address)
    }

    fn parse_maps_line(maps_line: &str) -> Option<ProcMapsRegion> {
        let mut line_tokens = maps_line.split_whitespace();
        let address_range_token = line_tokens.next()?;
        let permissions_token = line_tokens.next()?;
        // Skip offset, device, inode tokens.
        line_tokens.next()?;
        line_tokens.next()?;
        line_tokens.next()?;

        let pathname = line_tokens.collect::<Vec<_>>().join(" ");

        let (start_address_token, end_address_token) = address_range_token.split_once('-')?;
        let start_address = u64::from_str_radix(start_address_token, 16).ok()?;
        let end_address = u64::from_str_radix(end_address_token, 16).ok()?;

        if end_address <= start_address {
            return None;
        }

        Some(ProcMapsRegion {
            start_address,
            end_address,
            permissions: permissions_token.to_string(),
            pathname,
        })
    }

    fn parse_smaps_kilobyte_field(
        smaps_line: &str,
        field_name: &str,
    ) -> Option<u64> {
        let field_value = smaps_line.trim().strip_prefix(field_name)?.trim();
        let kilobytes_token = field_value.split_whitespace().next()?;
        let kilobytes = kilobytes_token.parse::<u64>().ok()?;

        Some(kilobytes.saturating_mul(1024))
    }

    fn parse_smaps_vm_flags(smaps_line: &str) -> Option<HashSet<String>> {
        let flags_value = smaps_line.trim().strip_prefix("VmFlags:")?.trim();

        Some(flags_value.split_whitespace().map(str::to_string).collect())
    }

    fn parse_protection_flags(permissions: &str) -> MemoryProtectionEnum {
        let mut protection_flags = MemoryProtectionEnum::empty();
        let permission_chars: Vec<char> = permissions.chars().collect();

        if permission_chars.first() == Some(&'r') {
            protection_flags |= MemoryProtectionEnum::READ;
        }

        if permission_chars.get(1) == Some(&'w') {
            protection_flags |= MemoryProtectionEnum::WRITE;
        }

        if permission_chars.get(2) == Some(&'x') {
            protection_flags |= MemoryProtectionEnum::EXECUTE;
        }

        // Linux private mappings are copy-on-write by design.
        if permission_chars.get(3) == Some(&'p') {
            protection_flags |= MemoryProtectionEnum::COPY_ON_WRITE;
        }

        protection_flags
    }

    fn parse_memory_type_flags(region: &ProcMapsRegion) -> MemoryTypeEnum {
        let mut type_flags = MemoryTypeEnum::empty();
        let is_private_mapping = region.permissions.chars().nth(3) == Some('p');
        let is_executable_mapping = region.permissions.chars().nth(2) == Some('x');

        if region.pathname.is_empty() {
            type_flags |= MemoryTypeEnum::NONE;
        }

        if is_private_mapping {
            type_flags |= MemoryTypeEnum::PRIVATE;
        } else {
            type_flags |= MemoryTypeEnum::MAPPED;
        }

        if is_executable_mapping && region.pathname.starts_with('/') {
            type_flags |= MemoryTypeEnum::IMAGE;
        }

        if type_flags.is_empty() {
            type_flags |= MemoryTypeEnum::NONE;
        }

        type_flags
    }

    fn matches_protection_filters(
        protection_flags: MemoryProtectionEnum,
        required_protection_bits: u32,
        excluded_protection_bits: u32,
    ) -> bool {
        let protection_bits = protection_flags.bits();

        if !protection_flags.contains(MemoryProtectionEnum::READ) {
            return false;
        }

        if required_protection_bits != 0 && (protection_bits & required_protection_bits) == 0 {
            return false;
        }

        if excluded_protection_bits != 0 && (protection_bits & excluded_protection_bits) != 0 {
            return false;
        }

        true
    }

    fn matches_type_filters(
        type_flags: MemoryTypeEnum,
        allowed_type_bits: u32,
    ) -> bool {
        if allowed_type_bits == 0 {
            return true;
        }

        (type_flags.bits() & allowed_type_bits) != 0
    }

    fn is_android_device_mapping(pathname: &str) -> bool {
        let pathname = pathname.trim();

        pathname.starts_with("/dev/") || pathname.starts_with("/dmabuf:")
    }

    fn is_platform_scan_candidate(region: &ProcMapsRegion) -> bool {
        #[cfg(target_os = "android")]
        {
            !Self::is_android_device_mapping(&region.pathname)
        }

        #[cfg(not(target_os = "android"))]
        {
            let _ = region;
            true
        }
    }

    fn get_platform_smaps_region_info_by_start_address(process_id: u32) -> Option<HashMap<u64, SmapsRegionInfo>> {
        #[cfg(target_os = "android")]
        {
            Self::parse_proc_smaps(process_id).ok()
        }

        #[cfg(not(target_os = "android"))]
        {
            let _ = process_id;
            None
        }
    }

    fn matches_platform_smaps_filters(
        region: &ProcMapsRegion,
        smaps_region_info_by_start_address: Option<&HashMap<u64, SmapsRegionInfo>>,
    ) -> bool {
        #[cfg(target_os = "android")]
        {
            let Some(smaps_region_info) =
                smaps_region_info_by_start_address.and_then(|region_info_by_start_address| region_info_by_start_address.get(&region.start_address))
            else {
                return true;
            };

            if smaps_region_info.rss_bytes == 0 {
                return false;
            }

            !smaps_region_info.vm_flags.contains("io") && !smaps_region_info.vm_flags.contains("pf")
        }

        #[cfg(not(target_os = "android"))]
        {
            let _ = region;
            let _ = smaps_region_info_by_start_address;
            true
        }
    }

    fn clamp_region_to_bounds(
        region: &ProcMapsRegion,
        start_address: u64,
        end_address: u64,
        region_bounds_handling: RegionBoundsHandling,
    ) -> Option<NormalizedRegion> {
        if region.end_address <= start_address || region.start_address >= end_address {
            return None;
        }

        match region_bounds_handling {
            RegionBoundsHandling::Exclude => {
                if region.start_address >= start_address && region.end_address <= end_address {
                    Some(NormalizedRegion::new(
                        region.start_address,
                        region.end_address.saturating_sub(region.start_address),
                    ))
                } else {
                    None
                }
            }
            RegionBoundsHandling::Include => Some(NormalizedRegion::new(
                region.start_address,
                region.end_address.saturating_sub(region.start_address),
            )),
            RegionBoundsHandling::Resize => {
                let clamped_start_address = region.start_address.max(start_address);
                let clamped_end_address = region.end_address.min(end_address);

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

    fn normalize_module_path(pathname: &str) -> String {
        pathname
            .strip_suffix(" (deleted)")
            .unwrap_or(pathname)
            .to_string()
    }

    fn is_module_region(region: &ProcMapsRegion) -> bool {
        region.permissions.chars().nth(2) == Some('x') && region.pathname.starts_with('/')
    }

    fn collect_module_paths(parsed_regions: &[ProcMapsRegion]) -> HashSet<String> {
        parsed_regions
            .iter()
            .filter(|parsed_region| Self::is_module_region(parsed_region))
            .map(|parsed_region| Self::normalize_module_path(&parsed_region.pathname))
            .collect()
    }

    fn build_modules_from_regions(parsed_regions: &[ProcMapsRegion]) -> Vec<NormalizedModule> {
        let module_paths = Self::collect_module_paths(parsed_regions);
        let mut module_ranges: HashMap<String, (u64, u64)> = HashMap::new();

        for parsed_region in parsed_regions {
            let module_path = Self::normalize_module_path(&parsed_region.pathname);

            if !module_paths.contains(&module_path) {
                continue;
            }

            let module_range_entry = module_ranges
                .entry(module_path)
                .or_insert((parsed_region.start_address, parsed_region.end_address));

            module_range_entry.0 = module_range_entry.0.min(parsed_region.start_address);
            module_range_entry.1 = module_range_entry.1.max(parsed_region.end_address);
        }

        let mut modules: Vec<NormalizedModule> = module_ranges
            .iter()
            .filter_map(|(module_path, (module_start_address, module_end_address))| {
                let module_region_size = module_end_address.saturating_sub(*module_start_address);
                if module_region_size == 0 {
                    return None;
                }

                let module_name = Self::module_name_from_path(module_path);

                Some(NormalizedModule::new(&module_name, *module_start_address, module_region_size))
            })
            .collect();

        modules.sort_by_key(|module| module.get_base_address());

        modules
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

impl MemoryQueryerTrait for LinuxMemoryQueryer {
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

        let parsed_regions = match Self::parse_proc_maps(process_info.get_process_id_raw()) {
            Ok(parsed_regions) => parsed_regions,
            Err(_) => return Vec::new(),
        };
        let required_protection_bits = required_protection.bits();
        let excluded_protection_bits = excluded_protection.bits();
        let allowed_type_bits = allowed_types.bits();
        let smaps_region_info_by_start_address = Self::get_platform_smaps_region_info_by_start_address(process_info.get_process_id_raw());

        parsed_regions
            .iter()
            .filter_map(|parsed_region| {
                let protection_flags = Self::parse_protection_flags(&parsed_region.permissions);
                if !Self::matches_protection_filters(protection_flags, required_protection_bits, excluded_protection_bits) {
                    return None;
                }

                let type_flags = Self::parse_memory_type_flags(parsed_region);
                if !Self::matches_type_filters(type_flags, allowed_type_bits) {
                    return None;
                }

                if !Self::is_platform_scan_candidate(parsed_region) {
                    return None;
                }

                if !Self::matches_platform_smaps_filters(parsed_region, smaps_region_info_by_start_address.as_ref()) {
                    return None;
                }

                Self::clamp_region_to_bounds(parsed_region, start_address, end_address, region_bounds_handling)
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
        let parsed_regions = match Self::parse_proc_maps(process_info.get_process_id_raw()) {
            Ok(parsed_regions) => parsed_regions,
            Err(_) => return false,
        };

        parsed_regions.iter().any(|parsed_region| {
            address >= parsed_region.start_address
                && address < parsed_region.end_address
                && Self::parse_protection_flags(&parsed_region.permissions).contains(MemoryProtectionEnum::WRITE)
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
        _process_info: &OpenedProcessInfo,
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
        let parsed_regions = match Self::parse_proc_maps(process_info.get_process_id_raw()) {
            Ok(parsed_regions) => parsed_regions,
            Err(_) => return Vec::new(),
        };

        Self::build_modules_from_regions(&parsed_regions)
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

#[cfg(test)]
mod tests {
    use super::{LinuxMemoryQueryer, ProcMapsRegion};
    use crate::memory_queryer::memory_protection_enum::MemoryProtectionEnum;
    use crate::memory_queryer::memory_queryer_trait::MemoryQueryerTrait;
    use crate::memory_queryer::memory_type_enum::MemoryTypeEnum;
    use crate::memory_queryer::region_bounds_handling::RegionBoundsHandling;
    use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;

    #[test]
    fn parse_maps_line_parses_well_formed_rows() {
        let maps_line = "7f12345000-7f12346000 r-xp 00000000 08:02 12345 /usr/lib/libexample.so";

        let parsed_region = LinuxMemoryQueryer::parse_maps_line(maps_line).expect("maps line should parse successfully.");

        assert_eq!(parsed_region.start_address, 0x7f12345000);
        assert_eq!(parsed_region.end_address, 0x7f12346000);
        assert_eq!(parsed_region.permissions, "r-xp");
        assert_eq!(parsed_region.pathname, "/usr/lib/libexample.so");
    }

    #[test]
    fn parse_maps_line_rejects_invalid_rows() {
        assert!(LinuxMemoryQueryer::parse_maps_line("not-a-maps-line").is_none());
        assert!(LinuxMemoryQueryer::parse_maps_line("1000-1000 r-xp 0 0 0 /tmp/file").is_none());
    }

    #[test]
    fn parse_protection_flags_maps_read_write_execute_and_private_bits() {
        let protection_flags = LinuxMemoryQueryer::parse_protection_flags("rwxp");

        assert!(protection_flags.contains(MemoryProtectionEnum::READ));
        assert!(protection_flags.contains(MemoryProtectionEnum::WRITE));
        assert!(protection_flags.contains(MemoryProtectionEnum::EXECUTE));
        assert!(protection_flags.contains(MemoryProtectionEnum::COPY_ON_WRITE));
    }

    #[test]
    fn matches_protection_filters_rejects_unreadable_regions() {
        let protection_flags = MemoryProtectionEnum::WRITE | MemoryProtectionEnum::COPY_ON_WRITE;

        assert!(!LinuxMemoryQueryer::matches_protection_filters(
            protection_flags,
            MemoryProtectionEnum::empty().bits(),
            MemoryProtectionEnum::empty().bits(),
        ));
    }

    #[test]
    fn android_device_mapping_filter_rejects_device_paths_only() {
        assert!(LinuxMemoryQueryer::is_android_device_mapping("/dev/mali0"));
        assert!(LinuxMemoryQueryer::is_android_device_mapping("/dmabuf:VRI[CaravanActivity]#4(BLAST Consumer)"));

        assert!(!LinuxMemoryQueryer::is_android_device_mapping(""));
        assert!(!LinuxMemoryQueryer::is_android_device_mapping("[heap]"));
        assert!(!LinuxMemoryQueryer::is_android_device_mapping("[anon:dalvik-LinearAlloc]"));
        assert!(!LinuxMemoryQueryer::is_android_device_mapping("/memfd:jit-cache (deleted)"));
        assert!(!LinuxMemoryQueryer::is_android_device_mapping("[anon:scudo:primary]"));
    }

    #[test]
    fn parse_smaps_fields_extracts_residency_and_vm_flags() {
        let rss_bytes = LinuxMemoryQueryer::parse_smaps_kilobyte_field("Rss:                123 kB", "Rss:");
        let vm_flags = LinuxMemoryQueryer::parse_smaps_vm_flags("VmFlags: rd wr mr mw me io pf");

        assert_eq!(rss_bytes, Some(123 * 1024));
        assert!(
            vm_flags
                .as_ref()
                .is_some_and(|vm_flags| vm_flags.contains("io") && vm_flags.contains("pf"))
        );
    }

    #[test]
    fn parse_memory_type_flags_marks_image_for_executable_file_mappings() {
        let parsed_region = ProcMapsRegion {
            start_address: 0x1000,
            end_address: 0x2000,
            permissions: "r-xp".to_string(),
            pathname: "/usr/bin/bash".to_string(),
        };

        let memory_type_flags = LinuxMemoryQueryer::parse_memory_type_flags(&parsed_region);

        assert!(memory_type_flags.contains(MemoryTypeEnum::IMAGE));
        assert!(memory_type_flags.contains(MemoryTypeEnum::PRIVATE));
    }

    #[test]
    fn clamp_region_to_bounds_resizes_intersecting_region() {
        let parsed_region = ProcMapsRegion {
            start_address: 0x1000,
            end_address: 0x3000,
            permissions: "rw-p".to_string(),
            pathname: String::new(),
        };

        let normalized_region = LinuxMemoryQueryer::clamp_region_to_bounds(&parsed_region, 0x1800, 0x2800, RegionBoundsHandling::Resize)
            .expect("resize bounds mode should keep the overlapping segment.");

        assert_eq!(normalized_region.get_base_address(), 0x1800);
        assert_eq!(normalized_region.get_region_size(), 0x1000);
    }

    #[test]
    fn resolve_module_matches_identifier_case_insensitively() {
        let queryer = LinuxMemoryQueryer::new();
        let modules = vec![NormalizedModule::new("libc.so.6", 0x1000, 0x5000)];

        let resolved_address = queryer.resolve_module(&modules, "LiBc.So.6");

        assert_eq!(resolved_address, 0x1000);
    }

    #[test]
    fn address_to_module_returns_module_name_and_offset() {
        let queryer = LinuxMemoryQueryer::new();
        let modules = vec![NormalizedModule::new("target.so", 0x4000, 0x2000)];

        let resolution = queryer.address_to_module(0x4ABC, &modules);

        let (module_name, module_offset) = resolution.expect("address should resolve inside target module.");
        assert_eq!(module_name, "target.so");
        assert_eq!(module_offset, 0xABC);
    }

    #[test]
    fn build_modules_from_regions_spans_all_mappings_for_executable_file() {
        let parsed_regions = vec![
            ProcMapsRegion {
                start_address: 0x1000,
                end_address: 0x2000,
                permissions: "r--p".to_string(),
                pathname: "/tmp/squalr".to_string(),
            },
            ProcMapsRegion {
                start_address: 0x2000,
                end_address: 0x5000,
                permissions: "r-xp".to_string(),
                pathname: "/tmp/squalr".to_string(),
            },
            ProcMapsRegion {
                start_address: 0x5000,
                end_address: 0x6000,
                permissions: "rw-p".to_string(),
                pathname: "/tmp/squalr".to_string(),
            },
            ProcMapsRegion {
                start_address: 0x8000,
                end_address: 0x9000,
                permissions: "rw-p".to_string(),
                pathname: "/tmp/data.bin".to_string(),
            },
        ];

        let modules = LinuxMemoryQueryer::build_modules_from_regions(&parsed_regions);

        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].get_module_name(), "squalr");
        assert_eq!(modules[0].get_base_address(), 0x1000);
        assert_eq!(modules[0].get_region_size(), 0x5000);

        let queryer = LinuxMemoryQueryer::new();
        let (_module_name, module_offset) = queryer
            .address_to_module(0x5800, &modules)
            .expect("address in writable executable-file mapping should resolve to the module.");

        assert_eq!(module_offset, 0x4800);
    }
}
