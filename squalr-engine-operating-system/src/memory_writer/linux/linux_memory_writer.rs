use crate::memory_writer::memory_writer_trait::MemoryWriterTrait;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
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

impl MemoryWriterTrait for LinuxMemoryWriter {
    fn write_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &[u8],
    ) -> bool {
        Self::write_memory(process_info.get_handle(), address, values)
    }
}
