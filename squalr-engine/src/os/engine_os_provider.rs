use squalr_engine_api::structures::data_values::data_value::DataValue;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::processes::process_info::ProcessInfo;
use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
use squalr_engine_memory::memory_queryer::memory_queryer::MemoryQueryer;
use squalr_engine_memory::memory_queryer::memory_queryer_trait::IMemoryQueryer;
use squalr_engine_memory::memory_queryer::page_retrieval_mode::PageRetrievalMode;
use squalr_engine_memory::memory_reader::MemoryReader;
use squalr_engine_memory::memory_reader::memory_reader_trait::IMemoryReader;
use squalr_engine_memory::memory_writer::MemoryWriter;
use squalr_engine_memory::memory_writer::memory_writer_trait::IMemoryWriter;
use squalr_engine_processes::process_query::process_query_error::ProcessQueryError;
use squalr_engine_processes::process_query::process_query_options::ProcessQueryOptions;
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
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
