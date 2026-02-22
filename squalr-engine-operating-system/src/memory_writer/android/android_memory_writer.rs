use crate::memory_writer::memory_writer_trait::MemoryWriterTrait;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use std::fs::OpenOptions;
use std::os::unix::fs::FileExt;

pub struct AndroidMemoryWriter;

impl AndroidMemoryWriter {
    pub fn new() -> Self {
        AndroidMemoryWriter
    }

    fn write_process_memory(
        process_id: u32,
        destination_address: u64,
        source_bytes: &[u8],
    ) -> bool {
        if source_bytes.is_empty() {
            return true;
        }

        let process_memory_path = format!("/proc/{process_id}/mem");
        let process_memory_file = match OpenOptions::new().write(true).open(process_memory_path) {
            Ok(process_memory_file) => process_memory_file,
            Err(error) => {
                log::error!("Failed to open process memory for write: {}", error);
                return false;
            }
        };

        let mut total_bytes_written: usize = 0;
        while total_bytes_written < source_bytes.len() {
            let next_write_offset = destination_address + total_bytes_written as u64;
            match process_memory_file.write_at(&source_bytes[total_bytes_written..], next_write_offset) {
                Ok(0) => return false,
                Ok(bytes_written) => total_bytes_written += bytes_written,
                Err(error) => {
                    log::error!("Failed to write process memory: {}", error);
                    return false;
                }
            }
        }

        true
    }
}

impl MemoryWriterTrait for AndroidMemoryWriter {
    fn write_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &[u8],
    ) -> bool {
        Self::write_process_memory(process_info.get_process_id_raw(), address, values)
    }
}
