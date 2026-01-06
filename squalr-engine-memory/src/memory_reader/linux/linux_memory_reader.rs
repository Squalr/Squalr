use crate::memory_reader::memory_reader_trait::IMemoryReader;

use squalr_engine_api::structures::data_values::data_value::DataValue;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;

use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

pub struct LinuxMemoryReader;

impl LinuxMemoryReader {
    pub fn new() -> Self {
        LinuxMemoryReader
    }

    fn read_proc_mem(
        pid: u32,
        address: u64,
        buffer: &mut [u8],
    ) -> Result<usize, Box<dyn Error>> {
        let mut mem_file = File::open(format!("/proc/{}/mem", pid))?;
        mem_file.seek(SeekFrom::Start(address))?;

        let bytes_read = mem_file.read(buffer)?;

        Ok(bytes_read)
    }
}

impl IMemoryReader for LinuxMemoryReader {
    fn read(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        data_value: &mut DataValue,
    ) -> bool {
        let size = data_value.get_size_in_bytes() as usize;
        let mut buffer = vec![0u8; size];

        if let Err(_) = Self::read_proc_mem(process_info.get_process_id_raw(), address, &mut buffer) {
            return false;
        }

        data_value.copy_from_bytes(&buffer);

        true
    }

    fn read_struct(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        valued_struct: &mut ValuedStruct,
    ) -> bool {
        let size = valued_struct.get_size_in_bytes() as usize;
        let mut buffer = vec![0u8; size];

        if let Err(_) = Self::read_proc_mem(process_info.get_process_id_raw(), address, &mut buffer) {
            return false;
        }

        valued_struct.copy_from_bytes(&buffer);

        true
    }

    fn read_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &mut [u8],
    ) -> bool {
        if let Err(_) = Self::read_proc_mem(process_info.get_process_id_raw(), address, values) {
            return false;
        }

        true
    }
}
