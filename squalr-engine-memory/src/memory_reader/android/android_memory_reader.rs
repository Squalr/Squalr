use crate::memory_reader::memory_reader_trait::IMemoryReader;
use squalr_engine_common::dynamic_struct::dynamic_struct::DynamicStruct;
use std::os::raw::c_void;

pub struct AndroidMemoryReader;

impl AndroidMemoryReader {
    pub fn new() -> Self {
        AndroidMemoryReader
    }
}

impl IMemoryReader for AndroidMemoryReader {
    fn read(
        &self,
        process_handle: u64,
        address: u64,
        dynamic_struct: &mut DynamicStruct,
    ) -> bool {
        false
    }

    fn read_bytes(
        &self,
        process_handle: u64,
        address: u64,
        values: &mut [u8],
    ) -> bool {
        false
    }
}
