use crate::memory_reader::memory_reader_trait::MemoryReaderTrait;
use squalr_engine_api::structures::data_values::data_value::DataValue;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
use std::fs::File;
use std::os::unix::fs::FileExt;

pub struct AndroidMemoryReader;

impl AndroidMemoryReader {
    pub fn new() -> Self {
        AndroidMemoryReader
    }

    fn read_process_memory(
        process_id: u32,
        source_address: u64,
        destination_buffer: &mut [u8],
    ) -> bool {
        if destination_buffer.is_empty() {
            return true;
        }

        let process_memory_path = format!("/proc/{process_id}/mem");
        let process_memory_file = match File::open(process_memory_path) {
            Ok(process_memory_file) => process_memory_file,
            Err(error) => {
                log::error!("Failed to open process memory for read: {}", error);
                return false;
            }
        };

        let mut total_bytes_read: usize = 0;
        while total_bytes_read < destination_buffer.len() {
            let next_read_offset = source_address + total_bytes_read as u64;
            match process_memory_file.read_at(&mut destination_buffer[total_bytes_read..], next_read_offset) {
                Ok(0) => return false,
                Ok(bytes_read) => total_bytes_read += bytes_read,
                Err(error) => {
                    log::error!("Failed to read process memory: {}", error);
                    return false;
                }
            }
        }

        true
    }
}

impl MemoryReaderTrait for AndroidMemoryReader {
    fn read(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        data_value: &mut DataValue,
    ) -> bool {
        let mut value_bytes = vec![0_u8; data_value.get_size_in_bytes() as usize];
        let read_succeeded = Self::read_process_memory(process_info.get_process_id_raw(), address, &mut value_bytes);

        if read_succeeded {
            data_value.copy_from_bytes(&value_bytes);
        }

        read_succeeded
    }

    fn read_struct(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        valued_struct: &mut ValuedStruct,
    ) -> bool {
        let mut struct_bytes = vec![0_u8; valued_struct.get_size_in_bytes() as usize];
        let read_succeeded = Self::read_process_memory(process_info.get_process_id_raw(), address, &mut struct_bytes);

        if read_succeeded {
            return valued_struct.copy_from_bytes(&struct_bytes);
        }

        false
    }

    fn read_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &mut [u8],
    ) -> bool {
        Self::read_process_memory(process_info.get_process_id_raw(), address, values)
    }
}
