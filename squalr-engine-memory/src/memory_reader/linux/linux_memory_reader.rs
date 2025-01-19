use crate::memory_reader::memory_reader_trait::IMemoryReader;
use squalr_engine_common::dynamic_struct::dynamic_struct::DynamicStruct;
use squalr_engine_processes::process_info::OpenedProcessInfo;
use std::os::raw::c_void;

pub struct LinuxMemoryReader;

impl LinuxMemoryReader {
    pub fn new() -> Self {
        LinuxMemoryReader
    }
}

impl IMemoryReader for LinuxMemoryReader {
    fn read(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        dynamic_struct: &mut DynamicStruct,
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
