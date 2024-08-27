use crate::memory_queryer::memory_protection_enum::MemoryProtectionEnum;
use crate::memory_queryer::memory_type_enum::MemoryTypeEnum;
use crate::memory_queryer::region_bounds_handling::RegionBoundsHandling;
use crate::memory_queryer::MemoryQueryerImpl;
use crate::normalized_region::NormalizedRegion;
use crate::{memory_queryer::memory_queryer_trait::IMemoryQueryer, memory_settings::MemorySettings};
use squalr_engine_common::logging::{log_level::LogLevel, logger::Logger};
use squalr_engine_processes::process_info::ProcessInfo;
use std::{collections::HashSet, sync::Once};

bitflags::bitflags! {
    #[derive(PartialEq, Eq)]
    pub struct PageRetrievalMode: u32 {
        const FROM_SETTINGS         = 1 << 0;
        const FROM_USER_MODE_MEMORY = 1 << 1;
        const FROM_NON_MODULES      = 1 << 2;
        const FROM_MODULES          = 1 << 3;
    }
}

pub struct MemoryQueryer;

impl MemoryQueryer {
    pub fn get_instance() -> &'static MemoryQueryerImpl {
        static mut INSTANCE: Option<MemoryQueryerImpl> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = MemoryQueryerImpl::new();
                INSTANCE = Some(instance);
            });

            return INSTANCE.as_ref().unwrap_unchecked();
        }
    }

    // TODO: Support middle-ware for emulator types to filter down the address space
    pub fn get_memory_page_bounds(
        process_info: &ProcessInfo,
        page_retrieval_mode: PageRetrievalMode,
    ) -> Vec<NormalizedRegion> {
        match page_retrieval_mode {
            PageRetrievalMode::FROM_SETTINGS => {
                return MemoryQueryer::query_pages_from_settings(process_info);
            }
            PageRetrievalMode::FROM_USER_MODE_MEMORY => {
                return MemoryQueryer::query_pages_from_usermode_memory(process_info);
            }
            PageRetrievalMode::FROM_MODULES => {
                return MemoryQueryer::query_pages_from_modules(process_info);
            }
            PageRetrievalMode::FROM_NON_MODULES => {
                return MemoryQueryer::query_pages_from_non_modules(process_info);
            }
            _ => {
                Logger::get_instance().log(LogLevel::Error, "Unknown snapshot retrieval mode", None);
                return vec![];
            }
        }
    }

    pub fn query_pages_by_address_range(
        process_info: &ProcessInfo,
        start_address: u64,
        end_address: u64,
    ) -> Vec<NormalizedRegion> {
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

        return normalized_regions;
    }

    fn query_pages_from_usermode_memory(process_info: &ProcessInfo) -> Vec<NormalizedRegion> {
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
            RegionBoundsHandling::Exclude,
        );

        return normalized_regions;
    }

    fn query_pages_from_settings(process_info: &ProcessInfo) -> Vec<NormalizedRegion> {
        let required_page_flags = MemoryQueryer::get_required_protection_settings();
        let excluded_page_flags = MemoryQueryer::get_excluded_protection_settings();
        let allowed_type_flags = MemoryQueryer::get_allowed_type_settings();

        let (start_address, end_address) = if MemorySettings::get_instance().is_usermode() {
            (0, MemoryQueryer::get_instance().get_max_usermode_address(process_info))
        } else {
            (
                MemorySettings::get_instance().get_start_address(),
                MemorySettings::get_instance().get_end_address(),
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

        return normalized_regions;
    }

    fn query_pages_from_modules(process_info: &ProcessInfo) -> Vec<NormalizedRegion> {
        // Note that we use into_base_region to extract the base region without copying, instead taking ownership
        let module_regions = MemoryQueryer::get_instance()
            .get_modules(process_info)
            .into_iter()
            .map(|module| module.into_base_region())
            .collect();

        return module_regions;
    }

    fn query_pages_from_non_modules(process_info: &ProcessInfo) -> Vec<NormalizedRegion> {
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
        let memory_regions = virtual_pages
            .into_iter()
            .filter(|page| !modules.contains(&page.get_base_address()))
            .collect();

        return memory_regions;
    }

    fn get_allowed_type_settings() -> MemoryTypeEnum {
        let mut result = MemoryTypeEnum::empty();

        if MemorySettings::get_instance().get_memory_type_none() {
            result |= MemoryTypeEnum::NONE;
        }

        if MemorySettings::get_instance().get_memory_type_private() {
            result |= MemoryTypeEnum::PRIVATE;
        }

        if MemorySettings::get_instance().get_memory_type_image() {
            result |= MemoryTypeEnum::IMAGE;
        }

        if MemorySettings::get_instance().get_memory_type_mapped() {
            result |= MemoryTypeEnum::MAPPED;
        }

        return result;
    }

    fn get_required_protection_settings() -> MemoryProtectionEnum {
        let mut result = MemoryProtectionEnum::empty();

        if MemorySettings::get_instance().get_required_write() {
            result |= MemoryProtectionEnum::WRITE;
        }

        if MemorySettings::get_instance().get_required_execute() {
            result |= MemoryProtectionEnum::EXECUTE;
        }

        if MemorySettings::get_instance().get_required_copy_on_write() {
            result |= MemoryProtectionEnum::COPY_ON_WRITE;
        }

        return result;
    }

    fn get_excluded_protection_settings() -> MemoryProtectionEnum {
        let mut result = MemoryProtectionEnum::empty();

        if MemorySettings::get_instance().get_excluded_write() {
            result |= MemoryProtectionEnum::WRITE;
        }

        if MemorySettings::get_instance().get_excluded_execute() {
            result |= MemoryProtectionEnum::EXECUTE;
        }

        if MemorySettings::get_instance().get_excluded_copy_on_write() {
            result |= MemoryProtectionEnum::COPY_ON_WRITE;
        }

        return result;
    }
}
