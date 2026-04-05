use crate::os::memory_view_router::MemoryViewRouter;
use crate::plugins::plugin_registry::PluginRegistry;
use squalr_engine_api::plugins::memory_view::MemoryViewPluginError;
use squalr_engine_api::structures::data_values::data_value::DataValue;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::processes::process_info::ProcessInfo;
use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
use squalr_engine_operating_system::memory_queryer::memory_queryer::MemoryQueryer;
use squalr_engine_operating_system::memory_queryer::memory_queryer_trait::MemoryQueryerTrait;
use squalr_engine_operating_system::memory_queryer::page_retrieval_mode::PageRetrievalMode;
use squalr_engine_operating_system::memory_reader::MemoryReader;
use squalr_engine_operating_system::memory_reader::memory_reader_trait::MemoryReaderTrait;
use squalr_engine_operating_system::memory_writer::MemoryWriter;
use squalr_engine_operating_system::memory_writer::memory_writer_trait::MemoryWriterTrait;
use squalr_engine_operating_system::process_query::process_query_error::ProcessQueryError;
use squalr_engine_operating_system::process_query::process_query_options::ProcessQueryOptions;
use squalr_engine_operating_system::process_query::process_queryer::ProcessQuery;
use std::collections::HashSet;
use std::sync::Arc;

pub trait ProcessQueryProvider: Send + Sync {
    fn start_monitoring(&self) -> Result<(), ProcessQueryError>;
    fn get_processes(
        &self,
        process_query_options: ProcessQueryOptions,
    ) -> Vec<ProcessInfo>;
    fn open_process(
        &self,
        process_info: &ProcessInfo,
    ) -> Result<OpenedProcessInfo, ProcessQueryError>;
    fn close_process(
        &self,
        handle: u64,
    ) -> Result<(), ProcessQueryError>;
}

pub trait MemoryQueryProvider: Send + Sync {
    fn get_modules(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Vec<NormalizedModule>;
    fn address_to_module(
        &self,
        address: u64,
        modules: &Vec<NormalizedModule>,
    ) -> Option<(String, u64)>;
    fn resolve_module(
        &self,
        modules: &Vec<NormalizedModule>,
        identifier: &str,
    ) -> u64;
    fn get_memory_page_bounds(
        &self,
        process_info: &OpenedProcessInfo,
        page_retrieval_mode: PageRetrievalMode,
    ) -> Vec<NormalizedRegion>;
}

pub trait MemoryReadProvider: Send + Sync {
    fn read(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        data_value: &mut DataValue,
    ) -> bool;
    fn read_struct(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        valued_struct: &mut ValuedStruct,
    ) -> bool;
    fn read_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &mut [u8],
    ) -> bool;
}

pub trait MemoryWriteProvider: Send + Sync {
    fn write_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &[u8],
    ) -> bool;
}

#[derive(Clone)]
pub struct EngineOsProviders {
    pub process_query: Arc<dyn ProcessQueryProvider>,
    pub memory_query: Arc<dyn MemoryQueryProvider>,
    pub memory_read: Arc<dyn MemoryReadProvider>,
    pub memory_write: Arc<dyn MemoryWriteProvider>,
}

impl EngineOsProviders {
    pub fn new(
        process_query: Arc<dyn ProcessQueryProvider>,
        memory_query: Arc<dyn MemoryQueryProvider>,
        memory_read: Arc<dyn MemoryReadProvider>,
        memory_write: Arc<dyn MemoryWriteProvider>,
    ) -> Self {
        Self {
            process_query,
            memory_query,
            memory_read,
            memory_write,
        }
    }

    pub fn with_memory_view_routing(
        self,
        plugin_registry: Arc<PluginRegistry>,
    ) -> Self {
        let memory_view_router = Arc::new(MemoryViewRouter::new(plugin_registry));
        let Self {
            process_query,
            memory_query,
            memory_read,
            memory_write,
        } = self;
        let base_memory_query = memory_query.clone();

        Self {
            process_query: Arc::new(RoutedProcessQueryProvider::new(process_query, memory_view_router.clone())),
            memory_query: Arc::new(RoutedMemoryQueryProvider::new(memory_query, memory_view_router.clone())),
            memory_read: Arc::new(RoutedMemoryReadProvider::new(
                memory_read,
                base_memory_query.clone(),
                memory_view_router.clone(),
            )),
            memory_write: Arc::new(RoutedMemoryWriteProvider::new(memory_write, base_memory_query, memory_view_router)),
        }
    }
}

impl Default for EngineOsProviders {
    fn default() -> Self {
        Self {
            process_query: Arc::new(DefaultProcessQueryProvider {}),
            memory_query: Arc::new(DefaultMemoryQueryProvider {}),
            memory_read: Arc::new(DefaultMemoryReadProvider {}),
            memory_write: Arc::new(DefaultMemoryWriteProvider {}),
        }
    }
}

struct DefaultProcessQueryProvider;

impl ProcessQueryProvider for DefaultProcessQueryProvider {
    fn start_monitoring(&self) -> Result<(), ProcessQueryError> {
        ProcessQuery::start_monitoring()
    }

    fn get_processes(
        &self,
        process_query_options: ProcessQueryOptions,
    ) -> Vec<ProcessInfo> {
        ProcessQuery::get_processes(process_query_options)
    }

    fn open_process(
        &self,
        process_info: &ProcessInfo,
    ) -> Result<OpenedProcessInfo, ProcessQueryError> {
        ProcessQuery::open_process(process_info)
    }

    fn close_process(
        &self,
        handle: u64,
    ) -> Result<(), ProcessQueryError> {
        ProcessQuery::close_process(handle)
    }
}

struct DefaultMemoryQueryProvider;

impl MemoryQueryProvider for DefaultMemoryQueryProvider {
    fn get_modules(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Vec<NormalizedModule> {
        MemoryQueryer::get_instance().get_modules(process_info)
    }

    fn address_to_module(
        &self,
        address: u64,
        modules: &Vec<NormalizedModule>,
    ) -> Option<(String, u64)> {
        MemoryQueryer::get_instance().address_to_module(address, modules)
    }

    fn resolve_module(
        &self,
        modules: &Vec<NormalizedModule>,
        identifier: &str,
    ) -> u64 {
        MemoryQueryer::get_instance().resolve_module(modules, identifier)
    }

    fn get_memory_page_bounds(
        &self,
        process_info: &OpenedProcessInfo,
        page_retrieval_mode: PageRetrievalMode,
    ) -> Vec<NormalizedRegion> {
        MemoryQueryer::get_memory_page_bounds(process_info, page_retrieval_mode)
    }
}

struct DefaultMemoryReadProvider;

impl MemoryReadProvider for DefaultMemoryReadProvider {
    fn read(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        data_value: &mut DataValue,
    ) -> bool {
        MemoryReader::get_instance().read(process_info, address, data_value)
    }

    fn read_struct(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        valued_struct: &mut ValuedStruct,
    ) -> bool {
        MemoryReader::get_instance().read_struct(process_info, address, valued_struct)
    }

    fn read_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &mut [u8],
    ) -> bool {
        MemoryReader::get_instance().read_bytes(process_info, address, values)
    }
}

struct DefaultMemoryWriteProvider;

impl MemoryWriteProvider for DefaultMemoryWriteProvider {
    fn write_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &[u8],
    ) -> bool {
        MemoryWriter::get_instance().write_bytes(process_info, address, values)
    }
}

struct RoutedProcessQueryProvider {
    base_provider: Arc<dyn ProcessQueryProvider>,
    memory_view_router: Arc<MemoryViewRouter>,
}

impl RoutedProcessQueryProvider {
    fn new(
        base_provider: Arc<dyn ProcessQueryProvider>,
        memory_view_router: Arc<MemoryViewRouter>,
    ) -> Self {
        Self {
            base_provider,
            memory_view_router,
        }
    }
}

impl ProcessQueryProvider for RoutedProcessQueryProvider {
    fn start_monitoring(&self) -> Result<(), ProcessQueryError> {
        self.base_provider.start_monitoring()
    }

    fn get_processes(
        &self,
        process_query_options: ProcessQueryOptions,
    ) -> Vec<ProcessInfo> {
        self.base_provider.get_processes(process_query_options)
    }

    fn open_process(
        &self,
        process_info: &ProcessInfo,
    ) -> Result<OpenedProcessInfo, ProcessQueryError> {
        self.base_provider.open_process(process_info)
    }

    fn close_process(
        &self,
        handle: u64,
    ) -> Result<(), ProcessQueryError> {
        let close_result = self.base_provider.close_process(handle);

        if close_result.is_ok() {
            self.memory_view_router.clear_cached_instance(handle);
        }

        close_result
    }
}

struct RoutedMemoryQueryProvider {
    base_provider: Arc<dyn MemoryQueryProvider>,
    memory_view_router: Arc<MemoryViewRouter>,
}

impl RoutedMemoryQueryProvider {
    fn new(
        base_provider: Arc<dyn MemoryQueryProvider>,
        memory_view_router: Arc<MemoryViewRouter>,
    ) -> Self {
        Self {
            base_provider,
            memory_view_router,
        }
    }

    fn merge_modules(
        &self,
        base_modules: Vec<NormalizedModule>,
        virtual_modules: Vec<NormalizedModule>,
    ) -> Vec<NormalizedModule> {
        let mut seen_module_keys = HashSet::new();
        let mut merged_modules = Vec::with_capacity(base_modules.len().saturating_add(virtual_modules.len()));

        for module in base_modules.into_iter().chain(virtual_modules.into_iter()) {
            let module_key = (module.get_module_name().to_string(), module.get_base_address(), module.get_region_size());

            if seen_module_keys.insert(module_key) {
                merged_modules.push(module);
            }
        }

        merged_modules
    }

    fn log_fallback_if_unexpected(
        plugin_id: &str,
        operation: &str,
        fallback_target: &str,
        error: &MemoryViewPluginError,
    ) {
        if error.is_unavailable() {
            return;
        }

        log::debug!(
            "Memory-view plugin `{}` {} fell back to base {}: {}",
            plugin_id,
            operation,
            fallback_target,
            error
        );
    }
}

impl MemoryQueryProvider for RoutedMemoryQueryProvider {
    fn get_modules(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Vec<NormalizedModule> {
        let base_modules = self.base_provider.get_modules(process_info);

        if let Some(memory_view_instance) = self.memory_view_router.get_or_create_instance(process_info) {
            match memory_view_instance.lock() {
                Ok(mut memory_view_instance) => {
                    if let Err(error) = memory_view_instance.refresh() {
                        Self::log_fallback_if_unexpected(memory_view_instance.plugin_id(), "refresh", "modules", &error);
                    } else {
                        match memory_view_instance.get_modules() {
                            Ok(virtual_modules) => return self.merge_modules(base_modules, virtual_modules),
                            Err(error) => {
                                Self::log_fallback_if_unexpected(memory_view_instance.plugin_id(), "module query", "modules", &error);
                            }
                        }
                    }
                }
                Err(error) => {
                    log::warn!("Failed to lock memory-view instance for module query: {}", error);
                }
            }
        }

        base_modules
    }

    fn address_to_module(
        &self,
        address: u64,
        modules: &Vec<NormalizedModule>,
    ) -> Option<(String, u64)> {
        if let Some(memory_view_instance) = self.memory_view_router.get_active_instance() {
            match memory_view_instance.lock() {
                Ok(memory_view_instance) => {
                    return memory_view_instance.address_to_module(address, modules);
                }
                Err(error) => {
                    log::warn!("Failed to lock memory-view instance for address-to-module resolution: {}", error);
                }
            }
        }

        self.base_provider.address_to_module(address, modules)
    }

    fn resolve_module(
        &self,
        modules: &Vec<NormalizedModule>,
        identifier: &str,
    ) -> u64 {
        if let Some(memory_view_instance) = self.memory_view_router.get_active_instance() {
            match memory_view_instance.lock() {
                Ok(memory_view_instance) => {
                    return memory_view_instance.resolve_module(modules, identifier);
                }
                Err(error) => {
                    log::warn!("Failed to lock memory-view instance for module resolution: {}", error);
                }
            }
        }

        self.base_provider.resolve_module(modules, identifier)
    }

    fn get_memory_page_bounds(
        &self,
        process_info: &OpenedProcessInfo,
        page_retrieval_mode: PageRetrievalMode,
    ) -> Vec<NormalizedRegion> {
        if let Some(memory_view_instance) = self.memory_view_router.get_or_create_instance(process_info) {
            match memory_view_instance.lock() {
                Ok(mut memory_view_instance) => {
                    if let Err(error) = memory_view_instance.refresh() {
                        RoutedMemoryQueryProvider::log_fallback_if_unexpected(memory_view_instance.plugin_id(), "refresh", "page bounds", &error);
                    } else {
                        match memory_view_instance.get_virtual_pages(page_retrieval_mode) {
                            Ok(regions) => return regions,
                            Err(error) => {
                                RoutedMemoryQueryProvider::log_fallback_if_unexpected(memory_view_instance.plugin_id(), "page query", "page bounds", &error);
                            }
                        }
                    }
                }
                Err(error) => {
                    log::warn!("Failed to lock memory-view instance for page query: {}", error);
                }
            }
        }

        self.base_provider
            .get_memory_page_bounds(process_info, page_retrieval_mode)
    }
}

struct RoutedMemoryReadProvider {
    base_provider: Arc<dyn MemoryReadProvider>,
    base_memory_query_provider: Arc<dyn MemoryQueryProvider>,
    memory_view_router: Arc<MemoryViewRouter>,
}

enum MemoryViewReadResult {
    Success,
    Failed,
    FallBackToBase,
}

impl RoutedMemoryReadProvider {
    fn new(
        base_provider: Arc<dyn MemoryReadProvider>,
        base_memory_query_provider: Arc<dyn MemoryQueryProvider>,
        memory_view_router: Arc<MemoryViewRouter>,
    ) -> Self {
        Self {
            base_provider,
            base_memory_query_provider,
            memory_view_router,
        }
    }

    fn is_range_mapped_by_base_provider(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        value_count: usize,
    ) -> bool {
        let range_size = (value_count as u64).max(1);
        let range_end_address = address.saturating_add(range_size.saturating_sub(1));

        self.base_memory_query_provider
            .get_memory_page_bounds(process_info, PageRetrievalMode::FromUserMode)
            .into_iter()
            .any(|region| {
                let region_start_address = region.get_base_address();
                let region_end_address = region_start_address.saturating_add(region.get_region_size().saturating_sub(1));

                address >= region_start_address && range_end_address <= region_end_address
            })
    }

    fn read_bytes_from_memory_view(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &mut [u8],
    ) -> MemoryViewReadResult {
        let Some(memory_view_instance) = self.memory_view_router.get_or_create_instance(process_info) else {
            return MemoryViewReadResult::FallBackToBase;
        };

        match memory_view_instance.lock() {
            Ok(memory_view_instance) => {
                let owns_address = memory_view_instance.owns_address(address);

                if !owns_address {
                    return MemoryViewReadResult::FallBackToBase;
                }

                match memory_view_instance.read_bytes(address, values) {
                    Ok(()) => MemoryViewReadResult::Success,
                    Err(_error) => {
                        if self.is_range_mapped_by_base_provider(process_info, address, values.len()) {
                            MemoryViewReadResult::FallBackToBase
                        } else {
                            MemoryViewReadResult::Failed
                        }
                    }
                }
            }
            Err(error) => {
                log::warn!("Failed to lock memory-view instance for read: {}", error);
                MemoryViewReadResult::FallBackToBase
            }
        }
    }
}

impl MemoryReadProvider for RoutedMemoryReadProvider {
    fn read(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        data_value: &mut DataValue,
    ) -> bool {
        let mut value_bytes = vec![0u8; data_value.get_size_in_bytes() as usize];

        match self.read_bytes_from_memory_view(process_info, address, &mut value_bytes) {
            MemoryViewReadResult::Success => {
                data_value.copy_from_bytes(&value_bytes);
                return true;
            }
            MemoryViewReadResult::Failed => return false,
            MemoryViewReadResult::FallBackToBase => {}
        }

        self.base_provider.read(process_info, address, data_value)
    }

    fn read_struct(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        valued_struct: &mut ValuedStruct,
    ) -> bool {
        let mut value_bytes = vec![0u8; valued_struct.get_size_in_bytes() as usize];

        match self.read_bytes_from_memory_view(process_info, address, &mut value_bytes) {
            MemoryViewReadResult::Success => return valued_struct.copy_from_bytes(&value_bytes),
            MemoryViewReadResult::Failed => return false,
            MemoryViewReadResult::FallBackToBase => {}
        }

        self.base_provider
            .read_struct(process_info, address, valued_struct)
    }

    fn read_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &mut [u8],
    ) -> bool {
        match self.read_bytes_from_memory_view(process_info, address, values) {
            MemoryViewReadResult::Success => return true,
            MemoryViewReadResult::Failed => return false,
            MemoryViewReadResult::FallBackToBase => {}
        }

        self.base_provider.read_bytes(process_info, address, values)
    }
}

struct RoutedMemoryWriteProvider {
    base_provider: Arc<dyn MemoryWriteProvider>,
    base_memory_query_provider: Arc<dyn MemoryQueryProvider>,
    memory_view_router: Arc<MemoryViewRouter>,
}

impl RoutedMemoryWriteProvider {
    fn new(
        base_provider: Arc<dyn MemoryWriteProvider>,
        base_memory_query_provider: Arc<dyn MemoryQueryProvider>,
        memory_view_router: Arc<MemoryViewRouter>,
    ) -> Self {
        Self {
            base_provider,
            base_memory_query_provider,
            memory_view_router,
        }
    }

    fn is_range_mapped_by_base_provider(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        value_count: usize,
    ) -> bool {
        let range_size = (value_count as u64).max(1);
        let range_end_address = address.saturating_add(range_size.saturating_sub(1));

        self.base_memory_query_provider
            .get_memory_page_bounds(process_info, PageRetrievalMode::FromUserMode)
            .into_iter()
            .any(|region| {
                let region_start_address = region.get_base_address();
                let region_end_address = region_start_address.saturating_add(region.get_region_size().saturating_sub(1));

                address >= region_start_address && range_end_address <= region_end_address
            })
    }
}

impl MemoryWriteProvider for RoutedMemoryWriteProvider {
    fn write_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &[u8],
    ) -> bool {
        if let Some(memory_view_instance) = self.memory_view_router.get_or_create_instance(process_info) {
            match memory_view_instance.lock() {
                Ok(memory_view_instance) => {
                    let owns_address = memory_view_instance.owns_address(address);

                    if !owns_address {
                        return self.base_provider.write_bytes(process_info, address, values);
                    }

                    match memory_view_instance.write_bytes(address, values) {
                        Ok(()) => return true,
                        Err(_error) => {
                            if self.is_range_mapped_by_base_provider(process_info, address, values.len()) {
                                return self.base_provider.write_bytes(process_info, address, values);
                            }

                            return false;
                        }
                    }
                }
                Err(error) => {
                    log::warn!("Failed to lock memory-view instance for write: {}", error);
                }
            }
        }

        self.base_provider.write_bytes(process_info, address, values)
    }
}

#[cfg(test)]
mod tests {
    use super::{EngineOsProviders, MemoryQueryProvider, MemoryReadProvider, MemoryWriteProvider, ProcessQueryProvider};
    use crate::plugins::plugin_registry::PluginRegistry;
    use squalr_engine_api::structures::{
        data_values::data_value::DataValue,
        memory::{bitness::Bitness, normalized_module::NormalizedModule, normalized_region::NormalizedRegion},
        processes::{opened_process_info::OpenedProcessInfo, process_info::ProcessInfo},
        structs::valued_struct::ValuedStruct,
    };
    use squalr_engine_operating_system::{
        memory_queryer::page_retrieval_mode::PageRetrievalMode,
        process_query::{process_query_error::ProcessQueryError, process_query_options::ProcessQueryOptions},
    };
    use std::sync::{Arc, Mutex};

    struct TestProcessQueryProvider;

    impl ProcessQueryProvider for TestProcessQueryProvider {
        fn start_monitoring(&self) -> Result<(), ProcessQueryError> {
            Ok(())
        }

        fn get_processes(
            &self,
            _process_query_options: ProcessQueryOptions,
        ) -> Vec<ProcessInfo> {
            vec![]
        }

        fn open_process(
            &self,
            _process_info: &ProcessInfo,
        ) -> Result<OpenedProcessInfo, ProcessQueryError> {
            Err(ProcessQueryError::internal("open_process", "not used in routing tests"))
        }

        fn close_process(
            &self,
            _handle: u64,
        ) -> Result<(), ProcessQueryError> {
            Ok(())
        }
    }

    struct TestMemoryQueryProvider {
        module_query_count: Arc<Mutex<usize>>,
        page_query_count: Arc<Mutex<usize>>,
        pages: Vec<NormalizedRegion>,
    }

    impl MemoryQueryProvider for TestMemoryQueryProvider {
        fn get_modules(
            &self,
            _process_info: &OpenedProcessInfo,
        ) -> Vec<NormalizedModule> {
            if let Ok(mut module_query_count) = self.module_query_count.lock() {
                *module_query_count += 1;
            }

            vec![NormalizedModule::new("game.exe", 0x1000, 0x200)]
        }

        fn address_to_module(
            &self,
            address: u64,
            modules: &Vec<NormalizedModule>,
        ) -> Option<(String, u64)> {
            modules
                .iter()
                .find(|module| module.contains_address(address))
                .map(|module| (module.get_module_name().to_string(), address.saturating_sub(module.get_base_address())))
        }

        fn resolve_module(
            &self,
            modules: &Vec<NormalizedModule>,
            identifier: &str,
        ) -> u64 {
            modules
                .iter()
                .find(|module| module.get_module_name().eq_ignore_ascii_case(identifier))
                .map(|module| module.get_base_address())
                .unwrap_or(0)
        }

        fn get_memory_page_bounds(
            &self,
            _process_info: &OpenedProcessInfo,
            _page_retrieval_mode: PageRetrievalMode,
        ) -> Vec<NormalizedRegion> {
            if let Ok(mut page_query_count) = self.page_query_count.lock() {
                *page_query_count += 1;
            }

            self.pages.clone()
        }
    }

    struct TestMemoryReadProvider {
        read_bytes_count: Arc<Mutex<usize>>,
    }

    impl MemoryReadProvider for TestMemoryReadProvider {
        fn read(
            &self,
            _process_info: &OpenedProcessInfo,
            _address: u64,
            data_value: &mut DataValue,
        ) -> bool {
            if let Ok(mut read_bytes_count) = self.read_bytes_count.lock() {
                *read_bytes_count += 1;
            }

            data_value.copy_from_bytes(&[0xAA, 0xBB]);
            true
        }

        fn read_struct(
            &self,
            _process_info: &OpenedProcessInfo,
            _address: u64,
            _valued_struct: &mut ValuedStruct,
        ) -> bool {
            false
        }

        fn read_bytes(
            &self,
            _process_info: &OpenedProcessInfo,
            _address: u64,
            values: &mut [u8],
        ) -> bool {
            if let Ok(mut read_bytes_count) = self.read_bytes_count.lock() {
                *read_bytes_count += 1;
            }

            values.copy_from_slice(&vec![0x11; values.len()]);
            true
        }
    }

    struct TestMemoryWriteProvider {
        write_count: Arc<Mutex<usize>>,
    }

    impl MemoryWriteProvider for TestMemoryWriteProvider {
        fn write_bytes(
            &self,
            _process_info: &OpenedProcessInfo,
            _address: u64,
            _values: &[u8],
        ) -> bool {
            if let Ok(mut write_count) = self.write_count.lock() {
                *write_count += 1;
            }

            true
        }
    }

    #[test]
    fn memory_query_falls_back_to_base_provider_when_no_plugin_matches() {
        let module_query_count = Arc::new(Mutex::new(0));
        let page_query_count = Arc::new(Mutex::new(0));
        let os_providers = EngineOsProviders::new(
            Arc::new(TestProcessQueryProvider),
            Arc::new(TestMemoryQueryProvider {
                module_query_count: module_query_count.clone(),
                page_query_count: page_query_count.clone(),
                pages: vec![NormalizedRegion::new(0x1000, 0x80)],
            }),
            Arc::new(TestMemoryReadProvider {
                read_bytes_count: Arc::new(Mutex::new(0)),
            }),
            Arc::new(TestMemoryWriteProvider {
                write_count: Arc::new(Mutex::new(0)),
            }),
        )
        .with_memory_view_routing(Arc::new(PluginRegistry::new()));
        let opened_process_info = OpenedProcessInfo::new(7, String::from("notepad.exe"), 42, Bitness::Bit64, None);

        let modules = os_providers.memory_query.get_modules(&opened_process_info);
        let regions = os_providers
            .memory_query
            .get_memory_page_bounds(&opened_process_info, PageRetrievalMode::FromUserMode);

        assert_eq!(modules.len(), 1);
        assert_eq!(regions.len(), 1);
        assert_eq!(
            *module_query_count
                .lock()
                .expect("Expected module query count lock."),
            1
        );
        assert_eq!(
            *page_query_count
                .lock()
                .expect("Expected page query count lock."),
            1
        );
    }

    #[test]
    fn dolphin_plugin_stub_does_not_fall_back_for_unmapped_guest_addresses() {
        let page_query_count = Arc::new(Mutex::new(0));
        let read_bytes_count = Arc::new(Mutex::new(0));
        let write_count = Arc::new(Mutex::new(0));
        let os_providers = EngineOsProviders::new(
            Arc::new(TestProcessQueryProvider),
            Arc::new(TestMemoryQueryProvider {
                module_query_count: Arc::new(Mutex::new(0)),
                page_query_count: page_query_count.clone(),
                pages: vec![NormalizedRegion::new(0x1000, 0x80)],
            }),
            Arc::new(TestMemoryReadProvider {
                read_bytes_count: read_bytes_count.clone(),
            }),
            Arc::new(TestMemoryWriteProvider {
                write_count: write_count.clone(),
            }),
        )
        .with_memory_view_routing(Arc::new(PluginRegistry::new()));
        let opened_process_info = OpenedProcessInfo::new(7, String::from("Dolphin.exe"), 42, Bitness::Bit64, None);
        let mut read_bytes = [0u8; 2];

        let regions = os_providers
            .memory_query
            .get_memory_page_bounds(&opened_process_info, PageRetrievalMode::FromUserMode);
        let read_succeeded = os_providers
            .memory_read
            .read_bytes(&opened_process_info, 0x8000_0000, &mut read_bytes);
        let write_succeeded = os_providers
            .memory_write
            .write_bytes(&opened_process_info, 0x8000_0000, &[0x22, 0x33]);

        assert_eq!(regions.len(), 1);
        assert!(!read_succeeded);
        assert!(!write_succeeded);
        assert_eq!(read_bytes, [0x00, 0x00]);
        assert_eq!(
            *page_query_count
                .lock()
                .expect("Expected page query count lock."),
            3
        );
        assert_eq!(*read_bytes_count.lock().expect("Expected read byte count lock."), 0);
        assert_eq!(*write_count.lock().expect("Expected write count lock."), 0);
    }

    #[test]
    fn dolphin_plugin_stub_falls_back_to_base_for_host_mapped_guest_range_addresses() {
        let page_query_count = Arc::new(Mutex::new(0));
        let read_bytes_count = Arc::new(Mutex::new(0));
        let write_count = Arc::new(Mutex::new(0));
        let os_providers = EngineOsProviders::new(
            Arc::new(TestProcessQueryProvider),
            Arc::new(TestMemoryQueryProvider {
                module_query_count: Arc::new(Mutex::new(0)),
                page_query_count: page_query_count.clone(),
                pages: vec![NormalizedRegion::new(0x8000_0000, 0x80)],
            }),
            Arc::new(TestMemoryReadProvider {
                read_bytes_count: read_bytes_count.clone(),
            }),
            Arc::new(TestMemoryWriteProvider {
                write_count: write_count.clone(),
            }),
        )
        .with_memory_view_routing(Arc::new(PluginRegistry::new()));
        let opened_process_info = OpenedProcessInfo::new(7, String::from("Dolphin.exe"), 42, Bitness::Bit64, None);
        let mut read_bytes = [0u8; 2];

        let read_succeeded = os_providers
            .memory_read
            .read_bytes(&opened_process_info, 0x8000_0000, &mut read_bytes);
        let write_succeeded = os_providers
            .memory_write
            .write_bytes(&opened_process_info, 0x8000_0000, &[0x22, 0x33]);

        assert!(read_succeeded);
        assert!(write_succeeded);
        assert_eq!(read_bytes, [0x11, 0x11]);
        assert_eq!(
            *page_query_count
                .lock()
                .expect("Expected page query count lock."),
            2
        );
        assert_eq!(*read_bytes_count.lock().expect("Expected read byte count lock."), 1);
        assert_eq!(*write_count.lock().expect("Expected write count lock."), 1);
    }
}
