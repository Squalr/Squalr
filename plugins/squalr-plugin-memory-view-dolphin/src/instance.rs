use crate::{
    address_space::{
        DolphinMemoryRegionDescriptor, DolphinMemoryRegionKind, DolphinPointerScanDomain, find_dolphin_region_by_virtual_address, gc_wii_module_name,
        parse_gba_internal_ram_slot, parse_gba_work_ram_slot, resolve_virtual_address_from_module,
    },
    constants::DOLPHIN_PLUGIN_ID,
    discovery::discover_dolphin_memory_regions,
};
use squalr_engine_api::{
    plugins::memory_view::{MemoryViewInstance, MemoryViewPluginError, PageRetrievalMode},
    structures::{
        memory::{
            normalized_module::{ModuleAddressDisplay, NormalizedModule},
            normalized_region::NormalizedRegion,
        },
        processes::opened_process_info::OpenedProcessInfo,
    },
};
use squalr_engine_operating_system::{
    memory_reader::{MemoryReader, memory_reader_trait::MemoryReaderTrait},
    memory_writer::{MemoryWriter, memory_writer_trait::MemoryWriterTrait},
};
use std::{
    sync::RwLock,
    time::{Duration, Instant},
};

const DISCOVERY_REFRESH_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Default)]
struct DolphinDiscoveryCache {
    discovered_region_descriptors: Option<Vec<DolphinMemoryRegionDescriptor>>,
    unavailable_reason: Option<String>,
    last_refresh_at: Option<Instant>,
}

pub(crate) struct DolphinMemoryViewInstance {
    opened_process_info: OpenedProcessInfo,
    discovery_cache: RwLock<DolphinDiscoveryCache>,
}

impl DolphinMemoryViewInstance {
    pub(crate) fn new(process_info: OpenedProcessInfo) -> Self {
        Self {
            opened_process_info: process_info,
            discovery_cache: RwLock::new(DolphinDiscoveryCache::default()),
        }
    }

    fn get_or_discover_region_descriptors(&self) -> Result<Vec<DolphinMemoryRegionDescriptor>, MemoryViewPluginError> {
        if let Ok(discovery_cache) = self.discovery_cache.read() {
            if let Some(discovered_region_descriptors) = discovery_cache.discovered_region_descriptors.as_ref() {
                return Ok(discovered_region_descriptors.clone());
            }

            if let Some(unavailable_reason) = discovery_cache.unavailable_reason.as_ref() {
                return Err(MemoryViewPluginError::unavailable(self.plugin_id(), unavailable_reason.clone()));
            }
        }

        self.refresh_discovery_cache()?;

        if let Ok(discovery_cache) = self.discovery_cache.read() {
            if let Some(discovered_region_descriptors) = discovery_cache.discovered_region_descriptors.as_ref() {
                return Ok(discovered_region_descriptors.clone());
            }

            if let Some(unavailable_reason) = discovery_cache.unavailable_reason.as_ref() {
                return Err(MemoryViewPluginError::unavailable(self.plugin_id(), unavailable_reason.clone()));
            }
        }

        Err(MemoryViewPluginError::unavailable(
            self.plugin_id(),
            "no Dolphin memory regions are currently exposed".to_string(),
        ))
    }

    fn should_refresh_discovery_cache(&self) -> bool {
        match self.discovery_cache.read() {
            Ok(discovery_cache) => discovery_cache
                .last_refresh_at
                .map(|last_refresh_at| last_refresh_at.elapsed() >= DISCOVERY_REFRESH_INTERVAL)
                .unwrap_or(true),
            Err(_) => true,
        }
    }

    fn refresh_discovery_cache(&self) -> Result<(), MemoryViewPluginError> {
        if !self.should_refresh_discovery_cache() {
            return match self.discovery_cache.read() {
                Ok(discovery_cache) => {
                    if discovery_cache.discovered_region_descriptors.is_some() {
                        Ok(())
                    } else if let Some(unavailable_reason) = discovery_cache.unavailable_reason.as_ref() {
                        Err(MemoryViewPluginError::unavailable(self.plugin_id(), unavailable_reason.clone()))
                    } else {
                        Ok(())
                    }
                }
                Err(error) => Err(MemoryViewPluginError::message(
                    self.plugin_id(),
                    format!("failed to access Dolphin discovery cache: {}", error),
                )),
            };
        }

        let discovery_result = discover_dolphin_memory_regions(&self.opened_process_info);

        match self.discovery_cache.write() {
            Ok(mut discovery_cache) => {
                discovery_cache.last_refresh_at = Some(Instant::now());

                match discovery_result {
                    Ok(discovered_region_descriptors) => {
                        discovery_cache.discovered_region_descriptors = Some(discovered_region_descriptors);
                        discovery_cache.unavailable_reason = None;
                        Ok(())
                    }
                    Err(error) => {
                        discovery_cache.discovered_region_descriptors = None;
                        discovery_cache.unavailable_reason = if error.is_unavailable() { Some(error.to_string()) } else { None };
                        Err(error)
                    }
                }
            }
            Err(error) => Err(MemoryViewPluginError::message(
                self.plugin_id(),
                format!("failed to update Dolphin memory-region cache: {}", error),
            )),
        }
    }

    fn build_virtual_pages(
        &self,
        page_retrieval_mode: PageRetrievalMode,
        discovered_region_descriptors: &[DolphinMemoryRegionDescriptor],
    ) -> Vec<NormalizedRegion> {
        match page_retrieval_mode {
            PageRetrievalMode::FromNonModules => Vec::new(),
            PageRetrievalMode::FromSettings | PageRetrievalMode::FromUserMode | PageRetrievalMode::FromModules | PageRetrievalMode::FromVirtualModules => {
                discovered_region_descriptors
                    .iter()
                    .map(|region_descriptor| NormalizedRegion::new(region_descriptor.get_virtual_base_address(), region_descriptor.get_region_size()))
                    .collect()
            }
        }
    }

    fn build_modules(
        &self,
        discovered_region_descriptors: &[DolphinMemoryRegionDescriptor],
    ) -> Vec<NormalizedModule> {
        let mut modules = Vec::new();
        let has_gamecube_or_wii_regions = discovered_region_descriptors.iter().any(|region_descriptor| {
            matches!(
                region_descriptor.get_region_kind(),
                DolphinMemoryRegionKind::GameCubeMainMemory | DolphinMemoryRegionKind::WiiExtendedMemory
            )
        });
        let has_wii_region = discovered_region_descriptors
            .iter()
            .any(|region_descriptor| matches!(region_descriptor.get_region_kind(), DolphinMemoryRegionKind::WiiExtendedMemory));

        if has_gamecube_or_wii_regions {
            modules.push(NormalizedModule::new_with_display(
                gc_wii_module_name(),
                0x8000_0000,
                if has_wii_region { 0x0600_0000 } else { 0x0200_0000 },
                ModuleAddressDisplay::AbsoluteAddress,
            ));
        }

        modules.extend(
            discovered_region_descriptors
                .iter()
                .filter_map(|region_descriptor| match region_descriptor.get_region_kind() {
                    DolphinMemoryRegionKind::GameBoyAdvanceWorkRam(_) | DolphinMemoryRegionKind::GameBoyAdvanceInternalRam(_) => {
                        let module_name = region_descriptor.get_module_name();

                        Some(NormalizedModule::new(
                            &module_name,
                            region_descriptor.get_virtual_base_address(),
                            region_descriptor.get_region_size(),
                        ))
                    }
                    DolphinMemoryRegionKind::GameCubeMainMemory | DolphinMemoryRegionKind::WiiExtendedMemory => None,
                }),
        );

        modules
    }

    fn build_pointer_scan_virtual_pages(
        &self,
        discovered_region_descriptors: &[DolphinMemoryRegionDescriptor],
        target_address: u64,
    ) -> Vec<NormalizedRegion> {
        let Some(target_region_descriptor) = find_dolphin_region_by_virtual_address(discovered_region_descriptors, target_address) else {
            return self.build_virtual_pages(PageRetrievalMode::FromVirtualModules, discovered_region_descriptors);
        };

        let target_pointer_scan_domain = target_region_descriptor.get_pointer_scan_domain();

        discovered_region_descriptors
            .iter()
            .filter(|region_descriptor| region_descriptor.get_pointer_scan_domain() == target_pointer_scan_domain)
            .map(|region_descriptor| NormalizedRegion::new(region_descriptor.get_virtual_base_address(), region_descriptor.get_region_size()))
            .collect()
    }

    fn translate_virtual_range(
        &self,
        virtual_address: u64,
        value_count: usize,
    ) -> Result<u64, MemoryViewPluginError> {
        let discovered_region_descriptors = self.get_or_discover_region_descriptors()?;
        let Some(region_descriptor) = find_dolphin_region_by_virtual_address(&discovered_region_descriptors, virtual_address) else {
            return Err(MemoryViewPluginError::message(
                self.plugin_id(),
                format!("virtual address {:#X} is not within a discovered Dolphin memory region", virtual_address),
            ));
        };

        let Some(host_address) = region_descriptor.virtual_to_host_address(virtual_address) else {
            return Err(MemoryViewPluginError::message(
                self.plugin_id(),
                format!("virtual address {:#X} could not be translated to a host address", virtual_address),
            ));
        };
        let virtual_end_address = virtual_address.saturating_add(value_count as u64);
        let region_end_address = region_descriptor
            .get_virtual_base_address()
            .saturating_add(region_descriptor.get_region_size());

        if virtual_end_address > region_end_address {
            return Err(MemoryViewPluginError::message(
                self.plugin_id(),
                format!(
                    "virtual range {:#X}-{:#X} crosses the end of Dolphin region `{}`",
                    virtual_address,
                    virtual_end_address,
                    region_descriptor.get_module_name()
                ),
            ));
        }

        Ok(host_address)
    }
}

impl MemoryViewInstance for DolphinMemoryViewInstance {
    fn plugin_id(&self) -> &str {
        DOLPHIN_PLUGIN_ID
    }

    fn owns_address(
        &self,
        address: u64,
    ) -> bool {
        self.get_or_discover_region_descriptors()
            .map(|discovered_region_descriptors| {
                discovered_region_descriptors
                    .iter()
                    .any(|region_descriptor| region_descriptor.contains_virtual_address(address))
            })
            .unwrap_or(false)
    }

    fn refresh(&mut self) -> Result<(), MemoryViewPluginError> {
        self.refresh_discovery_cache()
    }

    fn get_virtual_pages(
        &self,
        page_retrieval_mode: PageRetrievalMode,
    ) -> Result<Vec<NormalizedRegion>, MemoryViewPluginError> {
        let discovered_region_descriptors = self.get_or_discover_region_descriptors()?;

        Ok(self.build_virtual_pages(page_retrieval_mode, &discovered_region_descriptors))
    }

    fn get_modules(&self) -> Result<Vec<NormalizedModule>, MemoryViewPluginError> {
        let discovered_region_descriptors = self.get_or_discover_region_descriptors()?;

        Ok(self.build_modules(&discovered_region_descriptors))
    }

    fn get_pointer_scan_memory_regions(
        &self,
        page_retrieval_mode: PageRetrievalMode,
        target_address: u64,
    ) -> Result<Option<Vec<NormalizedRegion>>, MemoryViewPluginError> {
        if page_retrieval_mode != PageRetrievalMode::FromVirtualModules {
            return Ok(None);
        }

        let discovered_region_descriptors = self.get_or_discover_region_descriptors()?;

        Ok(Some(self.build_pointer_scan_virtual_pages(&discovered_region_descriptors, target_address)))
    }

    fn address_to_module(
        &self,
        address: u64,
        modules: &[NormalizedModule],
    ) -> Option<(String, u64)> {
        if let Ok(discovered_region_descriptors) = self.get_or_discover_region_descriptors() {
            if let Some(region_descriptor) = find_dolphin_region_by_virtual_address(&discovered_region_descriptors, address) {
                return region_descriptor.module_relative_address(address);
            }
        }

        modules
            .iter()
            .find(|module| module.contains_address(address))
            .map(|module| (module.get_module_name().to_string(), address.saturating_sub(module.get_base_address())))
    }

    fn resolve_module(
        &self,
        modules: &[NormalizedModule],
        identifier: &str,
    ) -> u64 {
        if identifier.eq_ignore_ascii_case(gc_wii_module_name()) || identifier.eq_ignore_ascii_case("gc") || identifier.eq_ignore_ascii_case("mem1") {
            return 0x8000_0000;
        }

        if identifier.eq_ignore_ascii_case("wii") || identifier.eq_ignore_ascii_case("mem2") {
            return 0x9000_0000;
        }

        if parse_gba_work_ram_slot(identifier).is_some() || parse_gba_internal_ram_slot(identifier).is_some() {
            return resolve_virtual_address_from_module(identifier, 0).unwrap_or(0);
        }

        modules
            .iter()
            .find(|module| module.get_module_name().eq_ignore_ascii_case(identifier))
            .map(|module| module.get_base_address())
            .unwrap_or(0)
    }

    fn resolve_module_address(
        &self,
        modules: &[NormalizedModule],
        identifier: &str,
        offset: u64,
    ) -> Option<u64> {
        if let Some(virtual_address) = resolve_virtual_address_from_module(identifier, offset) {
            let discovered_region_descriptors = self.get_or_discover_region_descriptors().ok()?;
            let region_descriptor = find_dolphin_region_by_virtual_address(&discovered_region_descriptors, virtual_address)?;

            return match region_descriptor.get_pointer_scan_domain() {
                DolphinPointerScanDomain::GcWii => {
                    if identifier.eq_ignore_ascii_case(gc_wii_module_name())
                        || identifier.eq_ignore_ascii_case("gc")
                        || identifier.eq_ignore_ascii_case("mem1")
                        || identifier.eq_ignore_ascii_case("wii")
                        || identifier.eq_ignore_ascii_case("mem2")
                    {
                        Some(virtual_address)
                    } else {
                        None
                    }
                }
                DolphinPointerScanDomain::Gba(slot_index) => {
                    if parse_gba_work_ram_slot(identifier) == Some(slot_index) || parse_gba_internal_ram_slot(identifier) == Some(slot_index) {
                        Some(virtual_address)
                    } else {
                        None
                    }
                }
            };
        }

        modules
            .iter()
            .find(|module| module.get_module_name().eq_ignore_ascii_case(identifier))
            .and_then(|module| module.get_base_address().checked_add(offset))
    }

    fn read_bytes(
        &self,
        address: u64,
        values: &mut [u8],
    ) -> Result<(), MemoryViewPluginError> {
        let host_address = self.translate_virtual_range(address, values.len())?;
        let read_succeeded = MemoryReader::get_instance().read_bytes(&self.opened_process_info, host_address, values);

        if read_succeeded {
            Ok(())
        } else {
            Err(MemoryViewPluginError::message(
                self.plugin_id(),
                format!("failed to read Dolphin host memory at translated address {:#X}", host_address),
            ))
        }
    }

    fn write_bytes(
        &self,
        address: u64,
        values: &[u8],
    ) -> Result<(), MemoryViewPluginError> {
        let host_address = self.translate_virtual_range(address, values.len())?;
        let write_succeeded = MemoryWriter::get_instance().write_bytes(&self.opened_process_info, host_address, values);

        if write_succeeded {
            Ok(())
        } else {
            Err(MemoryViewPluginError::message(
                self.plugin_id(),
                format!("failed to write Dolphin host memory at translated address {:#X}", host_address),
            ))
        }
    }
}
