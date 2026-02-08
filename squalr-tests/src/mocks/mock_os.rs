use squalr_engine::os::engine_os_provider::{EngineOsProviders, MemoryQueryProvider, MemoryReadProvider, MemoryWriteProvider, ProcessQueryProvider};
use squalr_engine_api::structures::data_values::data_value::DataValue;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::processes::process_info::ProcessInfo;
use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
use squalr_engine_memory::memory_queryer::page_retrieval_mode::PageRetrievalMode;
use squalr_engine_processes::process_query::process_query_error::ProcessQueryError;
use squalr_engine_processes::process_query::process_query_options::ProcessQueryOptions;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct RecordedProcessQueryOptions {
    pub required_process_id: Option<u32>,
    pub search_name: Option<String>,
    pub require_windowed: bool,
    pub match_case: bool,
    pub fetch_icons: bool,
    pub limit: Option<u64>,
}

#[derive(Default)]
pub struct MockOsState {
    pub process_query_requests: Vec<RecordedProcessQueryOptions>,
    pub close_process_handles: Vec<u64>,
    pub open_process_requests: Vec<u32>,
    pub memory_read_addresses: Vec<u64>,
    pub memory_struct_read_addresses: Vec<u64>,
    pub memory_write_requests: Vec<(u64, Vec<u8>)>,
    pub processes: Vec<ProcessInfo>,
    pub opened_process_result: Option<OpenedProcessInfo>,
    pub modules: Vec<NormalizedModule>,
    pub memory_pages: Vec<NormalizedRegion>,
    pub write_success: bool,
    pub read_success: bool,
}

#[derive(Clone)]
pub struct MockEngineOs {
    state: Arc<Mutex<MockOsState>>,
}

impl MockEngineOs {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MockOsState {
                write_success: true,
                read_success: true,
                ..MockOsState::default()
            })),
        }
    }

    pub fn get_state(&self) -> Arc<Mutex<MockOsState>> {
        self.state.clone()
    }

    pub fn set_processes(
        &self,
        processes: Vec<ProcessInfo>,
    ) {
        if let Ok(mut state_guard) = self.state.lock() {
            state_guard.processes = processes;
        }
    }

    pub fn set_opened_process_result(
        &self,
        opened_process_result: Option<OpenedProcessInfo>,
    ) {
        if let Ok(mut state_guard) = self.state.lock() {
            state_guard.opened_process_result = opened_process_result;
        }
    }

    pub fn set_modules(
        &self,
        modules: Vec<NormalizedModule>,
    ) {
        if let Ok(mut state_guard) = self.state.lock() {
            state_guard.modules = modules;
        }
    }

    pub fn set_memory_pages(
        &self,
        memory_pages: Vec<NormalizedRegion>,
    ) {
        if let Ok(mut state_guard) = self.state.lock() {
            state_guard.memory_pages = memory_pages;
        }
    }

    pub fn set_write_success(
        &self,
        write_success: bool,
    ) {
        if let Ok(mut state_guard) = self.state.lock() {
            state_guard.write_success = write_success;
        }
    }

    pub fn set_read_success(
        &self,
        read_success: bool,
    ) {
        if let Ok(mut state_guard) = self.state.lock() {
            state_guard.read_success = read_success;
        }
    }

    pub fn create_providers(&self) -> EngineOsProviders {
        let process_provider = Arc::new(MockProcessQueryProvider { state: self.state.clone() });
        let memory_query_provider = Arc::new(MockMemoryQueryProvider { state: self.state.clone() });
        let memory_read_provider = Arc::new(MockMemoryReadProvider { state: self.state.clone() });
        let memory_write_provider = Arc::new(MockMemoryWriteProvider { state: self.state.clone() });

        EngineOsProviders::new(process_provider, memory_query_provider, memory_read_provider, memory_write_provider)
    }
}

struct MockProcessQueryProvider {
    state: Arc<Mutex<MockOsState>>,
}

impl ProcessQueryProvider for MockProcessQueryProvider {
    fn start_monitoring(&self) -> Result<(), ProcessQueryError> {
        Ok(())
    }

    fn get_processes(
        &self,
        process_query_options: ProcessQueryOptions,
    ) -> Vec<ProcessInfo> {
        match self.state.lock() {
            Ok(mut state_guard) => {
                state_guard
                    .process_query_requests
                    .push(RecordedProcessQueryOptions {
                        required_process_id: process_query_options
                            .required_process_id
                            .map(|process_id| process_id.as_u32()),
                        search_name: process_query_options.search_name.clone(),
                        require_windowed: process_query_options.require_windowed,
                        match_case: process_query_options.match_case,
                        fetch_icons: process_query_options.fetch_icons,
                        limit: process_query_options.limit,
                    });

                state_guard.processes.clone()
            }
            Err(_error) => Vec::new(),
        }
    }

    fn open_process(
        &self,
        process_info: &ProcessInfo,
    ) -> Result<OpenedProcessInfo, ProcessQueryError> {
        match self.state.lock() {
            Ok(mut state_guard) => {
                state_guard
                    .open_process_requests
                    .push(process_info.get_process_id_raw());

                match state_guard.opened_process_result.clone() {
                    Some(opened_process_info) => Ok(opened_process_info),
                    None => Err(ProcessQueryError::internal("open_process", "No mocked opened process result configured.")),
                }
            }
            Err(error) => Err(ProcessQueryError::internal(
                "open_process",
                format!("Failed to lock mock process query provider: {}", error),
            )),
        }
    }

    fn close_process(
        &self,
        handle: u64,
    ) -> Result<(), ProcessQueryError> {
        match self.state.lock() {
            Ok(mut state_guard) => {
                state_guard.close_process_handles.push(handle);
                Ok(())
            }
            Err(error) => Err(ProcessQueryError::internal(
                "close_process",
                format!("Failed to lock mock process query provider: {}", error),
            )),
        }
    }
}

struct MockMemoryQueryProvider {
    state: Arc<Mutex<MockOsState>>,
}

impl MemoryQueryProvider for MockMemoryQueryProvider {
    fn get_modules(
        &self,
        _process_info: &OpenedProcessInfo,
    ) -> Vec<NormalizedModule> {
        match self.state.lock() {
            Ok(state_guard) => state_guard
                .modules
                .iter()
                .map(|module| NormalizedModule::new(module.get_module_name(), module.get_base_address(), module.get_region_size()))
                .collect(),
            Err(_error) => Vec::new(),
        }
    }

    fn address_to_module(
        &self,
        address: u64,
        modules: &Vec<NormalizedModule>,
    ) -> Option<(String, u64)> {
        for module in modules {
            if module.contains_address(address) {
                return Some((module.get_module_name().to_string(), address.saturating_sub(module.get_base_address())));
            }
        }

        None
    }

    fn resolve_module(
        &self,
        modules: &Vec<NormalizedModule>,
        identifier: &str,
    ) -> u64 {
        for module in modules {
            if module.get_module_name().eq_ignore_ascii_case(identifier) {
                return module.get_base_address();
            }
        }

        0
    }

    fn get_memory_page_bounds(
        &self,
        _process_info: &OpenedProcessInfo,
        _page_retrieval_mode: PageRetrievalMode,
    ) -> Vec<NormalizedRegion> {
        match self.state.lock() {
            Ok(state_guard) => state_guard.memory_pages.clone(),
            Err(_error) => Vec::new(),
        }
    }
}

struct MockMemoryReadProvider {
    state: Arc<Mutex<MockOsState>>,
}

impl MemoryReadProvider for MockMemoryReadProvider {
    fn read(
        &self,
        _process_info: &OpenedProcessInfo,
        address: u64,
        _data_value: &mut DataValue,
    ) -> bool {
        match self.state.lock() {
            Ok(mut state_guard) => {
                state_guard.memory_read_addresses.push(address);
                state_guard.read_success
            }
            Err(_error) => false,
        }
    }

    fn read_struct(
        &self,
        _process_info: &OpenedProcessInfo,
        address: u64,
        _valued_struct: &mut ValuedStruct,
    ) -> bool {
        match self.state.lock() {
            Ok(mut state_guard) => {
                state_guard.memory_struct_read_addresses.push(address);
                state_guard.read_success
            }
            Err(_error) => false,
        }
    }

    fn read_bytes(
        &self,
        _process_info: &OpenedProcessInfo,
        _address: u64,
        _values: &mut [u8],
    ) -> bool {
        match self.state.lock() {
            Ok(state_guard) => state_guard.read_success,
            Err(_error) => false,
        }
    }
}

struct MockMemoryWriteProvider {
    state: Arc<Mutex<MockOsState>>,
}

impl MemoryWriteProvider for MockMemoryWriteProvider {
    fn write_bytes(
        &self,
        _process_info: &OpenedProcessInfo,
        address: u64,
        values: &[u8],
    ) -> bool {
        match self.state.lock() {
            Ok(mut state_guard) => {
                state_guard
                    .memory_write_requests
                    .push((address, values.to_vec()));
                state_guard.write_success
            }
            Err(_error) => false,
        }
    }
}
