use crate::memory_reader::memory_reader_trait::MemoryReaderTrait;
use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
use squalr_engine_api::structures::{data_values::data_value::DataValue, processes::opened_process_info::OpenedProcessInfo};

pub struct LinuxMemoryReader;

impl LinuxMemoryReader {
    pub fn new() -> Self {
        LinuxMemoryReader
    }
}

impl MemoryReaderTrait for LinuxMemoryReader {
    fn read(
        &self,
        _process_info: &OpenedProcessInfo,
        _address: u64,
        _data_value: &mut DataValue,
    ) -> bool {
        false
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
        _values: &mut [u8],
    ) -> bool {
        false
    }
}
