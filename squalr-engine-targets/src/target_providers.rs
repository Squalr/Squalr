use squalr_engine_api::plugins::memory_view::PageRetrievalMode;
use squalr_engine_api::structures::data_values::data_value::DataValue;
use squalr_engine_api::structures::memory::{normalized_module::NormalizedModule, normalized_region::NormalizedRegion};
use squalr_engine_api::structures::processes::{opened_process_info::OpenedProcessInfo, process_info::ProcessInfo};
use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;

use crate::process_query::{process_query_error::ProcessQueryError, process_query_options::ProcessQueryOptions};

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

    fn resolve_module_address(
        &self,
        modules: &Vec<NormalizedModule>,
        identifier: &str,
        offset: u64,
    ) -> Option<u64> {
        self.resolve_module(modules, identifier).checked_add(offset)
    }

    fn get_memory_page_bounds(
        &self,
        process_info: &OpenedProcessInfo,
        page_retrieval_mode: PageRetrievalMode,
    ) -> Vec<NormalizedRegion>;

    fn get_pointer_scan_memory_page_bounds(
        &self,
        process_info: &OpenedProcessInfo,
        page_retrieval_mode: PageRetrievalMode,
        target_address: Option<u64>,
    ) -> Vec<NormalizedRegion> {
        let _ = target_address;

        self.get_memory_page_bounds(process_info, page_retrieval_mode)
    }
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
