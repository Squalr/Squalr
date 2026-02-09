use crate::memory_queryer::memory_protection_enum::MemoryProtectionEnum;
use crate::memory_queryer::memory_queryer_trait::MemoryQueryerTrait;
use crate::memory_queryer::memory_type_enum::MemoryTypeEnum;
use crate::memory_queryer::page_retrieval_mode::PageRetrievalMode;
use crate::memory_queryer::region_bounds_handling::RegionBoundsHandling;
use crate::{config::memory_settings_config::MemorySettingsConfig, memory_queryer::MemoryQueryerImpl};
use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use std::{collections::HashSet, sync::Once};

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

            #[allow(static_mut_refs)]
            INSTANCE.as_ref().unwrap_unchecked()
        }
    }

    pub fn get_memory_page_bounds(
        process_info: &OpenedProcessInfo,
        page_retrieval_mode: PageRetrievalMode,
    ) -> Vec<NormalizedRegion> {
        match page_retrieval_mode {
            PageRetrievalMode::FromSettings => MemoryQueryer::query_pages_from_settings(process_info),
            PageRetrievalMode::FromUserMode => MemoryQueryer::query_pages_from_usermode_memory(process_info),
            PageRetrievalMode::FromModules => MemoryQueryer::query_pages_from_modules(process_info),
            PageRetrievalMode::FromNonModules => MemoryQueryer::query_pages_from_non_modules(process_info),
        }
    }

    pub fn query_pages_by_address_range(
        process_info: &OpenedProcessInfo,
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

        normalized_regions
    }

    fn query_pages_from_usermode_memory(process_info: &OpenedProcessInfo) -> Vec<NormalizedRegion> {
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

        normalized_regions
    }

    fn query_pages_from_settings(process_info: &OpenedProcessInfo) -> Vec<NormalizedRegion> {
        let required_page_flags = MemoryQueryer::get_required_protection_settings();
        let excluded_page_flags = MemoryQueryer::get_excluded_protection_settings();
        let allowed_type_flags = MemoryQueryer::get_allowed_type_settings();

        let (start_address, end_address) = if MemorySettingsConfig::get_only_query_usermode() {
            (0, MemoryQueryer::get_instance().get_max_usermode_address(process_info))
        } else {
            (MemorySettingsConfig::get_start_address(), MemorySettingsConfig::get_end_address())
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

        normalized_regions
    }

    fn query_pages_from_modules(process_info: &OpenedProcessInfo) -> Vec<NormalizedRegion> {
        // Note that we use into_base_region to extract the base region without copying, instead taking ownership
        let module_regions = MemoryQueryer::get_instance()
            .get_modules(process_info)
            .into_iter()
            .map(|module| module.into_base_region())
            .collect();

        module_regions
    }

    fn query_pages_from_non_modules(process_info: &OpenedProcessInfo) -> Vec<NormalizedRegion> {
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

        memory_regions
    }

    fn get_allowed_type_settings() -> MemoryTypeEnum {
        let mut result = MemoryTypeEnum::empty();

        if MemorySettingsConfig::get_memory_type_none() {
            result |= MemoryTypeEnum::NONE;
        }

        if MemorySettingsConfig::get_memory_type_private() {
            result |= MemoryTypeEnum::PRIVATE;
        }

        if MemorySettingsConfig::get_memory_type_image() {
            result |= MemoryTypeEnum::IMAGE;
        }

        if MemorySettingsConfig::get_memory_type_mapped() {
            result |= MemoryTypeEnum::MAPPED;
        }

        result
    }

    fn get_required_protection_settings() -> MemoryProtectionEnum {
        let mut result = MemoryProtectionEnum::empty();

        if MemorySettingsConfig::get_required_write() {
            result |= MemoryProtectionEnum::WRITE;
        }

        if MemorySettingsConfig::get_required_execute() {
            result |= MemoryProtectionEnum::EXECUTE;
        }

        if MemorySettingsConfig::get_required_copy_on_write() {
            result |= MemoryProtectionEnum::COPY_ON_WRITE;
        }

        result
    }

    fn get_excluded_protection_settings() -> MemoryProtectionEnum {
        let mut result = MemoryProtectionEnum::empty();

        if MemorySettingsConfig::get_excluded_write() {
            result |= MemoryProtectionEnum::WRITE;
        }

        if MemorySettingsConfig::get_excluded_execute() {
            result |= MemoryProtectionEnum::EXECUTE;
        }

        if MemorySettingsConfig::get_excluded_copy_on_write() {
            result |= MemoryProtectionEnum::COPY_ON_WRITE;
        }

        result
    }
}
