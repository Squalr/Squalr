use crate::{
    address_space::{DolphinMemoryRegionDescriptor, find_dolphin_region_by_guest_address},
    constants::DOLPHIN_PLUGIN_ID,
    discovery::discover_dolphin_memory_regions,
};
use squalr_engine_api::{
    plugins::memory_view::{MemoryViewInstance, MemoryViewPluginError, PageRetrievalMode},
    structures::{
        memory::{normalized_module::NormalizedModule, normalized_region::NormalizedRegion},
        processes::opened_process_info::OpenedProcessInfo,
    },
};
use squalr_engine_operating_system::{
    memory_reader::{MemoryReader, memory_reader_trait::MemoryReaderTrait},
    memory_writer::{MemoryWriter, memory_writer_trait::MemoryWriterTrait},
};
use std::sync::RwLock;

pub(crate) struct DolphinMemoryViewInstance {
    opened_process_info: OpenedProcessInfo,
    discovered_region_descriptors: RwLock<Option<Vec<DolphinMemoryRegionDescriptor>>>,
}

impl DolphinMemoryViewInstance {
    pub(crate) fn new(process_info: OpenedProcessInfo) -> Self {
        Self {
            opened_process_info: process_info,
            discovered_region_descriptors: RwLock::new(None),
        }
    }

    fn get_or_discover_region_descriptors(&self) -> Result<Vec<DolphinMemoryRegionDescriptor>, MemoryViewPluginError> {
        if let Ok(discovered_region_descriptors) = self.discovered_region_descriptors.read() {
            if let Some(discovered_region_descriptors) = discovered_region_descriptors.as_ref() {
                return Ok(discovered_region_descriptors.clone());
            }
        }

        let discovered_region_descriptors = discover_dolphin_memory_regions(&self.opened_process_info)?;

        if let Ok(mut cached_region_descriptors) = self.discovered_region_descriptors.write() {
            *cached_region_descriptors = Some(discovered_region_descriptors.clone());
        }

        Ok(discovered_region_descriptors)
    }

    fn build_virtual_pages(
        &self,
        page_retrieval_mode: PageRetrievalMode,
        discovered_region_descriptors: &[DolphinMemoryRegionDescriptor],
    ) -> Vec<NormalizedRegion> {
        match page_retrieval_mode {
            PageRetrievalMode::FromNonModules => Vec::new(),
            PageRetrievalMode::FromSettings | PageRetrievalMode::FromUserMode | PageRetrievalMode::FromModules => discovered_region_descriptors
                .iter()
                .map(|region_descriptor| NormalizedRegion::new(region_descriptor.get_guest_base_address(), region_descriptor.get_region_size()))
                .collect(),
        }
    }

    fn build_modules(
        &self,
        discovered_region_descriptors: &[DolphinMemoryRegionDescriptor],
    ) -> Vec<NormalizedModule> {
        discovered_region_descriptors
            .iter()
            .map(|region_descriptor| {
                NormalizedModule::new(
                    region_descriptor.get_module_name(),
                    region_descriptor.get_guest_base_address(),
                    region_descriptor.get_region_size(),
                )
            })
            .collect()
    }

    fn translate_guest_range(
        &self,
        guest_address: u64,
        value_count: usize,
    ) -> Result<u64, MemoryViewPluginError> {
        let discovered_region_descriptors = self.get_or_discover_region_descriptors()?;
        let Some(region_descriptor) = find_dolphin_region_by_guest_address(&discovered_region_descriptors, guest_address) else {
            return Err(MemoryViewPluginError::message(
                self.plugin_id(),
                format!("guest address {:#X} is not within a discovered Dolphin memory region", guest_address),
            ));
        };

        let Some(host_address) = region_descriptor.guest_to_host_address(guest_address) else {
            return Err(MemoryViewPluginError::message(
                self.plugin_id(),
                format!("guest address {:#X} could not be translated to a host address", guest_address),
            ));
        };
        let guest_end_address = guest_address.saturating_add(value_count as u64);
        let region_end_address = region_descriptor
            .get_guest_base_address()
            .saturating_add(region_descriptor.get_region_size());

        if guest_end_address > region_end_address {
            return Err(MemoryViewPluginError::message(
                self.plugin_id(),
                format!(
                    "guest range {:#X}-{:#X} crosses the end of Dolphin region `{}`",
                    guest_address,
                    guest_end_address,
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

    fn refresh(&mut self) -> Result<(), MemoryViewPluginError> {
        let discovered_region_descriptors = discover_dolphin_memory_regions(&self.opened_process_info)?;

        match self.discovered_region_descriptors.write() {
            Ok(mut cached_region_descriptors) => {
                *cached_region_descriptors = Some(discovered_region_descriptors);
                Ok(())
            }
            Err(error) => Err(MemoryViewPluginError::message(
                self.plugin_id(),
                format!("failed to update Dolphin memory-region cache: {}", error),
            )),
        }
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

    fn read_bytes(
        &self,
        address: u64,
        values: &mut [u8],
    ) -> Result<(), MemoryViewPluginError> {
        let host_address = self.translate_guest_range(address, values.len())?;
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
        let host_address = self.translate_guest_range(address, values.len())?;
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
