use crate::memory_writer::memory_writer_trait::MemoryWriterTrait;
use mach2::kern_return::KERN_SUCCESS;
use mach2::vm::mach_vm_write;
use mach2::vm_types::{mach_vm_address_t, vm_offset_t};
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use std::convert::TryFrom;

pub struct MacOsMemoryWriter;

impl MacOsMemoryWriter {
    pub fn new() -> Self {
        MacOsMemoryWriter
    }

    fn write_memory(
        process_handle: u64,
        address: u64,
        data: &[u8],
    ) -> bool {
        if data.is_empty() {
            return true;
        }

        let data_length = match u32::try_from(data.len()) {
            Ok(data_length) => data_length,
            Err(_) => return false,
        };

        let write_status = unsafe { mach_vm_write(process_handle as _, address as mach_vm_address_t, data.as_ptr() as vm_offset_t, data_length) };

        write_status == KERN_SUCCESS
    }
}

impl MemoryWriterTrait for MacOsMemoryWriter {
    fn write_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &[u8],
    ) -> bool {
        Self::write_memory(process_info.get_handle(), address, values)
    }
}
