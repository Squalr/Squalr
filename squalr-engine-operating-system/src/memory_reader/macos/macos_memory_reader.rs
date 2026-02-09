use crate::memory_reader::memory_reader_trait::MemoryReaderTrait;
use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
use squalr_engine_api::structures::{data_values::data_value::DataValue, processes::opened_process_info::OpenedProcessInfo};
use std::os::raw::c_void;

pub struct MacOsMemoryReader;

impl MacOsMemoryReader {
    pub fn new() -> Self {
        MacOsMemoryReader
    }
}

impl MemoryReaderTrait for MacOsMemoryReader {
    fn read(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        data_value: &mut DataValue,
    ) -> bool {
        false
    }

    fn read_struct(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        valued_struct: &mut ValuedStruct,
    ) -> bool {
        false
    }

    fn read_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &mut [u8],
    ) -> bool {
        false
    }
}
