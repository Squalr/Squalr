use crate::scan_settings::ScanSettings;
use crate::snapshots::snapshot::Snapshot;
use crate::snapshots::snapshot_region::SnapshotRegion;

use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_processes::process_info::ProcessInfo;
use squalr_engine_memory::memory_queryer::memory_protection_enum::MemoryProtectionEnum;
use squalr_engine_memory::memory_queryer::MemoryQueryer;
use squalr_engine_memory::memory_queryer::memory_queryer_trait::IMemoryQueryer;
use squalr_engine_memory::memory_queryer::memory_type_enum::MemoryTypeEnum;
use squalr_engine_memory::memory_queryer::region_bounds_handling::RegionBoundsHandling;
use std::collections::HashSet;

bitflags::bitflags! {
    #[derive(PartialEq, Eq)]
    pub struct SnapshotRetrievalMode: u32 {
        const FROM_SETTINGS         = 1 << 0;
        const FROM_USER_MODE_MEMORY = 1 << 1;
        const FROM_NON_MODULES      = 1 << 2;
        const FROM_STACK            = 1 << 3;
        const FROM_MODULES          = 1 << 4;
    }
}

pub struct SnapshotQueryer;

impl SnapshotQueryer {

    // TODO: Support middle-ware for emulator types to filter down the address space
    pub fn get_snapshot(
        process_info: &ProcessInfo,
        snapshot_creation_mode: SnapshotRetrievalMode,
    ) -> Snapshot {
        match snapshot_creation_mode {
            SnapshotRetrievalMode::FROM_SETTINGS => {
                SnapshotQueryer::create_snapshot_from_settings(process_info)
            }
            SnapshotRetrievalMode::FROM_USER_MODE_MEMORY => {
                SnapshotQueryer::create_snapshot_from_usermode_memory(process_info)
            }
            SnapshotRetrievalMode::FROM_MODULES => {
                SnapshotQueryer::create_snapshot_from_modules(process_info)
            }
            SnapshotRetrievalMode::FROM_NON_MODULES => {
                SnapshotQueryer::create_snapshot_from_non_modules(process_info)
            }
            SnapshotRetrievalMode::FROM_STACK => unimplemented!(),
            _ => {
                Logger::get_instance().log(LogLevel::Error, "Unknown snapshot retrieval mode", None);
                Snapshot::new(String::from(""), vec![])
            }
        }
    }

    pub fn create_snapshot_by_address_range(
        process_info: &ProcessInfo,
        start_address: u64,
        end_address: u64,
    ) -> Snapshot {
        let required_page_flags = MemoryProtectionEnum::empty();
        let excluded_page_flags = MemoryProtectionEnum::empty();
        let allowed_type_flags = MemoryTypeEnum::NONE | MemoryTypeEnum::PRIVATE | MemoryTypeEnum::IMAGE | MemoryTypeEnum::MAPPED;
        let bounds_handling = RegionBoundsHandling::Resize;
    
        let normalized_regions = MemoryQueryer::get_instance().get_virtual_pages(
            process_info,
            required_page_flags,
            excluded_page_flags,
            allowed_type_flags,
            start_address,
            end_address,
            bounds_handling,
        );
    
        let mut snapshot_regions = vec![];
        for normalized_region in normalized_regions {
            let mut snapshot_region = SnapshotRegion::new_from_normalized_region(normalized_region);
            snapshot_region.set_alignment(ScanSettings::get_instance().get_alignment());
            snapshot_regions.push(snapshot_region);
        }
    
        Snapshot::new(String::from(""), snapshot_regions)
    }
    

    fn create_snapshot_from_usermode_memory(process_info: &ProcessInfo) -> Snapshot {
        let required_page_flags = MemoryProtectionEnum::empty();
        let excluded_page_flags = MemoryProtectionEnum::empty();
        let allowed_type_flags = MemoryTypeEnum::NONE | MemoryTypeEnum::PRIVATE | MemoryTypeEnum::IMAGE;
        let start_address = 0;
        let end_address = MemoryQueryer::get_instance().get_max_usermode_address(process_info);
    
        let normalized_regions = MemoryQueryer::get_instance().get_virtual_pages(
            process_info,
            required_page_flags,
            excluded_page_flags,
            allowed_type_flags,
            start_address,
            end_address,
            RegionBoundsHandling::Exclude
        );
    
        let mut snapshot_regions = vec![];
        for normalized_region in normalized_regions {
            let mut snapshot_region = SnapshotRegion::new_from_normalized_region(normalized_region);
            snapshot_region.set_alignment(ScanSettings::get_instance().get_alignment());
            snapshot_regions.push(snapshot_region);
        }
    
        Snapshot::new(String::from(""), snapshot_regions)
    }
    

    fn create_snapshot_from_settings(process_info: &ProcessInfo) -> Snapshot {
        let required_page_flags = SnapshotQueryer::get_required_protection_settings();
        let excluded_page_flags = SnapshotQueryer::get_excluded_protection_settings();
        let allowed_type_flags = SnapshotQueryer::get_allowed_type_settings();
    
        let (start_address, end_address) = if ScanSettings::get_instance().is_usermode() {
            (0, MemoryQueryer::get_instance().get_max_usermode_address(process_info))
        } else {
            (
                ScanSettings::get_instance().get_start_address(),
                ScanSettings::get_instance().get_end_address(),
            )
        };
    
        let normalized_regions = MemoryQueryer::get_instance().get_virtual_pages(
            process_info,
            required_page_flags,
            excluded_page_flags,
            allowed_type_flags,
            start_address,
            end_address,
            RegionBoundsHandling::Exclude,
        );
    
        let mut snapshot_regions = vec![];
        for normalized_region in normalized_regions {
            let mut snapshot_region = SnapshotRegion::new_from_normalized_region(normalized_region);
            snapshot_region.set_alignment(ScanSettings::get_instance().get_alignment());
            snapshot_regions.push(snapshot_region);
        }
    
        Snapshot::new(String::from(""), snapshot_regions)
    }
    
    fn create_snapshot_from_modules(process_info: &ProcessInfo) -> Snapshot {
        // Note that we use into_base_region to extract the base region without copying, instead taking ownership
        let module_regions: Vec<SnapshotRegion> = MemoryQueryer::get_instance()
            .get_modules(process_info)
            .into_iter()
            .map(|module| SnapshotRegion::new_from_normalized_region(module.into_base_region()))
            .collect();
    
        Snapshot::new(String::from(""), module_regions)
    }    

    fn create_snapshot_from_non_modules(process_info: &ProcessInfo) -> Snapshot {
        let modules: HashSet<u64> = MemoryQueryer::get_instance()
            .get_modules(process_info)
            .into_iter()
            .map(|module| module.get_base_address())
            .collect();
    
        let required_page_flags = MemoryProtectionEnum::empty();
        let excluded_page_flags = MemoryProtectionEnum::empty();
        let allowed_type_flags = MemoryTypeEnum::NONE | MemoryTypeEnum::PRIVATE | MemoryTypeEnum::IMAGE;
        let start_address = 0;
        let end_address = MemoryQueryer::get_instance().get_max_usermode_address(process_info);
        
        // Collect all virtual pages
        let virtual_pages = MemoryQueryer::get_instance().get_virtual_pages(
            process_info,
            required_page_flags,
            excluded_page_flags,
            allowed_type_flags,
            start_address,
            end_address,
            RegionBoundsHandling::Exclude,
        );
        
        // Exclude any virtual pages that are also modules (static)
        let memory_regions: Vec<SnapshotRegion> = virtual_pages
            .into_iter()
            .filter(|page| !modules.contains(&page.get_base_address()))
            .map(|page| {
                let mut snapshot_region = SnapshotRegion::new_from_normalized_region(page);
                snapshot_region.set_alignment(ScanSettings::get_instance().get_alignment());
                return snapshot_region;
            })
            .collect();
    
        Snapshot::new(String::from(""), memory_regions)
    }
    
    fn get_allowed_type_settings() -> MemoryTypeEnum {
        let mut result = MemoryTypeEnum::empty();

        if ScanSettings::get_instance().get_memory_type_none() {
            result |= MemoryTypeEnum::NONE;
        }

        if ScanSettings::get_instance().get_memory_type_private() {
            result |= MemoryTypeEnum::PRIVATE;
        }

        if ScanSettings::get_instance().get_memory_type_image() {
            result |= MemoryTypeEnum::IMAGE;
        }

        if ScanSettings::get_instance().get_memory_type_mapped() {
            result |= MemoryTypeEnum::MAPPED;
        }

        return result;
    }

    fn get_required_protection_settings() -> MemoryProtectionEnum {
        let mut result = MemoryProtectionEnum::empty();

        if ScanSettings::get_instance().get_required_write() {
            result |= MemoryProtectionEnum::WRITE;
        }

        if ScanSettings::get_instance().get_required_execute() {
            result |= MemoryProtectionEnum::EXECUTE;
        }

        if ScanSettings::get_instance().get_required_copy_on_write() {
            result |= MemoryProtectionEnum::COPY_ON_WRITE;
        }

        return result;
    }

    fn get_excluded_protection_settings() -> MemoryProtectionEnum {
        let mut result = MemoryProtectionEnum::empty();

        if ScanSettings::get_instance().get_excluded_write() {
            result |= MemoryProtectionEnum::WRITE;
        }

        if ScanSettings::get_instance().get_excluded_execute() {
            result |= MemoryProtectionEnum::EXECUTE;
        }

        if ScanSettings::get_instance().get_excluded_copy_on_write() {
            result |= MemoryProtectionEnum::COPY_ON_WRITE;
        }

        return result;
    }
}
