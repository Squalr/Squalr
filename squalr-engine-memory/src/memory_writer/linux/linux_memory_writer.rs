use crate::memory_writer::memory_writer_trait::IMemoryWriter;

use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;

use std::fs::File;
use std::io::{Seek, SeekFrom, Write};

pub struct LinuxMemoryWriter;

impl LinuxMemoryWriter {
    pub fn new() -> Self {
        LinuxMemoryWriter
    }

    fn write_proc_mem(
        pid: u32,
        address: u64,
        buffer: &[u8],
    ) -> bool {
        let mut mem_file = match File::open(format!("/proc/{}/mem", pid)) {
            Ok(f) => f,
            Err(_) => return false
        };

        if let Err(_) = mem_file.seek(SeekFrom::Start(address)) {
            return false;
        }

        match mem_file.write(buffer) {
            Ok(_) => true
            Err(_) => false
        }
    }
}

impl IMemoryWriter for LinuxMemoryWriter {
    fn write_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &[u8],
    ) -> bool {
        Self::write_proc_mem(process_info.get_process_id_raw(), address, values)
    }
}
