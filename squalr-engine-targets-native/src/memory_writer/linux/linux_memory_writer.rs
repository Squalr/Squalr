use crate::memory_writer::memory_writer_trait::MemoryWriterTrait;
use libc::{c_void, iovec, pid_t, process_vm_writev};
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;

pub struct LinuxMemoryWriter;

impl LinuxMemoryWriter {
    pub fn new() -> Self {
        LinuxMemoryWriter
    }

    fn write_process_memory(
        process_id: u32,
        destination_address: u64,
        source_bytes: &[u8],
    ) -> bool {
        if source_bytes.is_empty() {
            return true;
        }

        let local_iovec = iovec {
            iov_base: source_bytes.as_ptr() as *mut c_void,
            iov_len: source_bytes.len(),
        };

        let remote_iovec = iovec {
            iov_base: destination_address as *mut c_void,
            iov_len: source_bytes.len(),
        };

        let bytes_written = unsafe { process_vm_writev(process_id as pid_t, &local_iovec, 1, &remote_iovec, 1, 0) };

        bytes_written == source_bytes.len() as isize
    }
}

impl MemoryWriterTrait for LinuxMemoryWriter {
    fn write_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &[u8],
    ) -> bool {
        Self::write_process_memory(process_info.get_process_id_raw(), address, values)
    }
}
