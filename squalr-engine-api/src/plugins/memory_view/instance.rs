use crate::{
    plugins::memory_view::{MemoryViewPluginError, PageRetrievalMode},
    structures::memory::{normalized_module::NormalizedModule, normalized_region::NormalizedRegion},
};

pub trait MemoryViewInstance: Send + Sync {
    fn plugin_id(&self) -> &str;

    fn owns_address(
        &self,
        _address: u64,
    ) -> bool {
        false
    }

    fn refresh(&mut self) -> Result<(), MemoryViewPluginError> {
        Ok(())
    }

    fn get_virtual_pages(
        &self,
        page_retrieval_mode: PageRetrievalMode,
    ) -> Result<Vec<NormalizedRegion>, MemoryViewPluginError>;

    fn get_modules(&self) -> Result<Vec<NormalizedModule>, MemoryViewPluginError>;

    fn address_to_module(
        &self,
        address: u64,
        modules: &[NormalizedModule],
    ) -> Option<(String, u64)> {
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
        self.resolve_module(modules, identifier).checked_add(offset)
    }

    fn get_pointer_scan_memory_regions(
        &self,
        _page_retrieval_mode: PageRetrievalMode,
        _target_address: u64,
    ) -> Result<Option<Vec<NormalizedRegion>>, MemoryViewPluginError> {
        Ok(None)
    }

    fn read_bytes(
        &self,
        address: u64,
        values: &mut [u8],
    ) -> Result<(), MemoryViewPluginError>;

    fn write_bytes(
        &self,
        address: u64,
        values: &[u8],
    ) -> Result<(), MemoryViewPluginError>;
}
