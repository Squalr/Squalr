use crate::memory_queryer::memory_protection_enum::MemoryProtectionEnum;
use crate::memory_queryer::memory_queryer_trait::IMemoryQueryer;
use crate::memory_queryer::memory_type_enum::MemoryTypeEnum;
use crate::memory_queryer::region_bounds_handling::RegionBoundsHandling;
use crate::normalized_module::NormalizedModule;
use crate::normalized_region::NormalizedRegion;
use squalr_engine_processes::process_info::OpenedProcessInfo;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct AndroidMemoryQueryer;

/// Helper struct to hold a single line of /proc/<pid>/maps data
struct ProcMapRegion {
    start: u64,
    end: u64,
    perms: String,
    offset: u64,
    dev: String,
    inode: u64,
    pathname: String,
}

impl AndroidMemoryQueryer {
    pub fn new() -> Self {
        AndroidMemoryQueryer
    }

    /// Reads `/proc/<pid>/maps` and returns a Vec of parsed `ProcMapRegion`.
    fn parse_proc_maps(pid: i32) -> std::io::Result<Vec<ProcMapRegion>> {
        let path = format!("/proc/{}/maps", pid);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut regions = Vec::new();

        for line_result in reader.lines() {
            let line = line_result?;
            // Example line format (fields are whitespace-delimited):
            // 00400000-00452000 r-xp 00000000 fc:01 1234   /system/bin/app_process32

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 5 {
                // At minimum we expect: address range, perms, offset, dev, inode, [pathname]
                continue;
            }

            // Address range "00400000-00452000"
            let range_part = parts[0];
            let perms_part = parts[1];
            let offset_part = parts[2];
            let dev_part = parts[3];
            let inode_part = parts[4];
            let pathname_part = if parts.len() > 5 { parts[5..].join(" ") } else { "".to_string() };

            // Parse start/end
            let mut range_split = range_part.split('-');
            let start_str = range_split.next().unwrap_or("0");
            let end_str = range_split.next().unwrap_or("0");

            let start = u64::from_str_radix(start_str, 16).unwrap_or(0);
            let end = u64::from_str_radix(end_str, 16).unwrap_or(0);
            let offset = u64::from_str_radix(offset_part, 16).unwrap_or(0);
            let inode = inode_part.parse::<u64>().unwrap_or(0);

            let region = ProcMapRegion {
                start,
                end,
                perms: perms_part.to_string(),
                offset,
                dev: dev_part.to_string(),
                inode,
                pathname: pathname_part,
            };

            regions.push(region);
        }

        Ok(regions)
    }

    /// Convert the 4-character perms (e.g. "r-xp") from /proc/maps into
    /// your custom `MemoryProtectionEnum`.
    fn to_memory_protection(perms: &str) -> MemoryProtectionEnum {
        // Typically perms is something like:
        //    "rwxp" => read, write, exec, private
        //    "r--s" => read, shared
        // We'll just map r/w/x to your bitflags and
        // treat p => private => might handle as well
        // (The user can interpret 'p' vs 's' as "private" vs "shared")
        let mut prot = MemoryProtectionEnum::empty();

        // If the string is at least 4 characters, we can check each one
        // index: 0 => read 'r'
        // index: 1 => write 'w'
        // index: 2 => execute 'x'
        // index: 3 => private/shared 'p'/'s'
        if perms.len() >= 1 && &perms[0..1] == "r" {
            prot |= MemoryProtectionEnum::READ;
        }
        if perms.len() >= 2 && &perms[1..2] == "w" {
            prot |= MemoryProtectionEnum::WRITE;
        }
        if perms.len() >= 3 && &perms[2..3] == "x" {
            prot |= MemoryProtectionEnum::EXECUTE;
        }
        // We'll skip COPY_ON_WRITE vs normal write distinction here
        // (some lines might have 'w' + 's' for shared writes, etc.)

        prot
    }

    /// Convert /proc/maps region to your `MemoryTypeEnum`.
    /// This is highly heuristic. You might decide based on the pathname,
    /// or whether 'p' vs. 's' is set, etc.
    fn to_memory_type(
        perms: &str,
        pathname: &str,
    ) -> MemoryTypeEnum {
        let mut mem_type = MemoryTypeEnum::empty();

        // If 'p' is at index 3 => private
        // If 's' is at index 3 => shared
        if perms.len() >= 4 && &perms[3..4] == "p" {
            mem_type |= MemoryTypeEnum::PRIVATE;
        } else {
            mem_type |= MemoryTypeEnum::MAPPED;
        }

        // If the pathname points to a .so or ELF, you might consider that IMAGE
        // Very naive check:
        if pathname.ends_with(".so") || pathname.contains(".so") {
            mem_type |= MemoryTypeEnum::IMAGE;
        }

        mem_type
    }

    /// Helper to see if a region meets the "required" and "excluded" protection criteria.
    fn match_protection(
        region_prot: &MemoryProtectionEnum,
        required: &MemoryProtectionEnum,
        excluded: &MemoryProtectionEnum,
    ) -> bool {
        /*
        // Region must contain all required bits
        if !required.is_empty() && !region_prot.contains(required) {
            return false;
        }
        // Region must not contain any excluded bits
        if !(region_prot & excluded).is_empty() {
            return false;
        } */
        true
    }

    /// Helper to see if a region's type meets the "allowed" type flags.
    fn match_type(
        region_type: &MemoryTypeEnum,
        allowed: &MemoryTypeEnum,
    ) -> bool {
        /*
        // If `allowed` is empty, that might mean the user wants none,
        // or it might mean "no restriction". Adjust as needed.
        // For now, let's say if allowed is non-empty, region_type
        // must share at least one bit with it (or must be fully contained).
        !(region_type & allowed).is_empty()
         */
        false
    }

    /// Adjust a region to the requested address range, if needed.
    /// This is only relevant if `region_bounds_handling == RegionBoundsHandling::Resize`.
    fn clamp_region(
        region: &ProcMapRegion,
        start_address: u64,
        end_address: u64,
        bounds_handling: RegionBoundsHandling,
    ) -> Option<(u64, u64)> {
        let rstart = region.start;
        let rend = region.end;
        if rstart >= end_address || rend <= start_address {
            // No overlap at all
            return match bounds_handling {
                RegionBoundsHandling::Exclude => None,
                RegionBoundsHandling::Resize => None,
                RegionBoundsHandling::Include => Some((rstart, rend)),
            };
        }

        // We do have an overlap
        match bounds_handling {
            RegionBoundsHandling::Exclude => {
                // If partial overlap, that might still be included,
                // but let's interpret "Exclude" to mean we must be fully in range
                if rstart < start_address || rend > end_address {
                    None
                } else {
                    Some((rstart, rend))
                }
            }
            RegionBoundsHandling::Resize => {
                // Clip to the intersection
                let clipped_start = rstart.max(start_address);
                let clipped_end = rend.min(end_address);
                if clipped_start < clipped_end {
                    Some((clipped_start, clipped_end))
                } else {
                    None
                }
            }
            RegionBoundsHandling::Include => {
                // Keep the entire region as-is, even if it partially extends
                Some((rstart, rend))
            }
        }
    }
}

impl IMemoryQueryer for AndroidMemoryQueryer {
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
        let pid_i32 = process_info.pid.as_u32() as i32;
        // or adapt if your `Pid` wrapper can convert to i32 differently

        let regions_result = Self::parse_proc_maps(pid_i32);
        let Ok(regions) = regions_result else {
            return vec![];
        };

        let mut out = Vec::new();

        for reg in regions {
            // Convert perms -> MemoryProtectionEnum
            let protection = Self::to_memory_protection(&reg.perms);
            if !Self::match_protection(&protection, &required_protection, &excluded_protection) {
                continue;
            }

            // Convert perms + pathname -> MemoryTypeEnum
            let mem_type = Self::to_memory_type(&reg.perms, &reg.pathname);
            if !Self::match_type(&mem_type, &allowed_types) {
                continue;
            }

            // Adjust region to the requested [start_address, end_address]
            if let Some((clamped_start, clamped_end)) = Self::clamp_region(&reg, start_address, end_address, region_bounds_handling) {
                let size = clamped_end - clamped_start;
                if size == 0 {
                    continue;
                }
                out.push(NormalizedRegion::new(clamped_start, size));
            }
        }

        out
    }

    /// Return all pages, ignoring user-provided start/end, filters, etc.
    fn get_all_virtual_pages(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Vec<NormalizedRegion> {
        // We can just call get_virtual_pages with minimal restrictions
        self.get_virtual_pages(
            process_info,
            MemoryProtectionEnum::empty(),
            MemoryProtectionEnum::empty(),
            MemoryTypeEnum::all(),
            0,
            std::u64::MAX,
            RegionBoundsHandling::Include,
        )
    }

    fn is_address_writable(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
    ) -> bool {
        // Naive approach: parse maps, find the region that contains `address`,
        // then see if it has WRITE bit. (This is O(n) each time. For real usage,
        // you might want a better indexing structure.)
        // zcanann: Normalized regions do not store protection info, can't do this.
        /*
        let pages = self.get_all_virtual_pages(process_info);
        for r in pages {
            let start = r.get_base_address();
            let end = start + r.get_region_size();
            if address >= start && address < end {
                return r.protection.contains(MemoryProtectionEnum::WRITE);
            }
        } */
        false
    }

    /// The maximum possible virtual address for a given architecture.
    /// On 64-bit Android, this could be up to the canonical 47-bit range, etc.
    /// We'll just return something large.
    fn get_maximum_address(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> u64 {
        match process_info.bitness {
            squalr_engine_processes::process_info::Bitness::Bit64 => 0x7FFFFFFFFFFF,
            squalr_engine_processes::process_info::Bitness::Bit32 => 0xFFFFFFFF,
        }
    }

    /// Minimum usermode address typically is 0 on Linux-based systems.
    fn get_min_usermode_address(
        &self,
        _process_info: &OpenedProcessInfo,
    ) -> u64 {
        0
    }

    /// Just returns something below the typical kernel boundary. For 64-bit
    /// Android, something like 0x7F_FFFF_FFFF might be feasible, or you can
    /// parse from /proc/<pid>/maps to find an actual maximum userland mapping.
    fn get_max_usermode_address(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> u64 {
        self.get_maximum_address(process_info)
    }

    /// "Modules" on Android can be considered any mapped .so or the executable
    /// lines from /proc/<pid>/maps. This is a naive approach: we gather all
    /// regions that have 'x' in perms and a non-empty pathname, etc.
    fn get_modules(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Vec<NormalizedModule> {
        let pid_i32 = process_info.pid.as_u32() as i32;
        let regions_result = Self::parse_proc_maps(pid_i32);
        let Ok(regions) = regions_result else {
            return vec![];
        };

        let mut modules = Vec::new();
        for reg in regions {
            // If perms has 'x' (execute) and pathname is not empty:
            if reg.perms.len() >= 3 && &reg.perms[2..3] == "x" && !reg.pathname.is_empty() {
                let size = reg.end.saturating_sub(reg.start);

                // Create a NormalizedModule
                modules.push(NormalizedModule::new(&reg.pathname, reg.start, size));
            }
        }
        modules
    }

    /// Given an address, find which module contains it. On Linux,
    /// you typically do the same parse of /proc/<pid>/maps or your cached modules.
    fn address_to_module(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        module_name: &mut String,
    ) -> u64 {
        let mods = self.get_modules(process_info);
        for m in mods {
            let start = m.get_base_address();
            let end = start + m.get_region_size();
            if address >= start && address < end {
                *module_name = m.get_name().to_string();
                return start;
            }
        }
        0
    }

    /// Attempt to find the module whose name or path matches `identifier`.
    /// Return its base address.
    fn resolve_module(
        &self,
        process_info: &OpenedProcessInfo,
        identifier: &str,
    ) -> u64 {
        let mods = self.get_modules(process_info);
        for m in mods {
            // naive substring match
            if m.get_name().contains(identifier) || m.get_full_path().contains(identifier) {
                return m.get_base_address();
            }
        }
        0
    }
}
