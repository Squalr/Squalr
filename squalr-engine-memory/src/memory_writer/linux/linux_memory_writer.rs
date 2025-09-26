use crate::memory_writer::memory_writer_trait::IMemoryWriter;
use squalr_engine_common::dynamic_struct::to_bytes::ToBytes;
use std::os::raw::c_void;
use std::ptr::null_mut;

pub struct LinuxMemoryWriter;

impl LinuxMemoryWriter {
    pub fn new() -> Self {
        LinuxMemoryWriter
    }

    fn write_memory(
        process_handle: u64,
        address: u64,
        data: &[u8],
    ) -> bool {
        false
    }
}

impl IMemoryWriter for LinuxMemoryWriter {
    fn write(
        &self,
        process_handle: u64,
        address: u64,
        value: &dyn ToBytes,
    ) -> bool {
        let bytes = value.to_bytes();
        Self::write_memory(process_handle, address, &bytes)
    }

    fn write_bytes(
        &self,
        process_handle: u64,
        address: u64,
        values: &[u8],
    ) -> bool {
        Self::write_memory(process_handle, address, values)
    }
}
