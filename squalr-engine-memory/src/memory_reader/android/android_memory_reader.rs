use crate::memory_reader::memory_reader_trait::IMemoryReader;
use squalr_engine_common::dynamic_struct::dynamic_struct::DynamicStruct;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_processes::process_info::OpenedProcessInfo;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom};

pub struct AndroidMemoryReader;

impl AndroidMemoryReader {
    pub fn new() -> Self {
        AndroidMemoryReader
    }

    fn read_bytes_internal(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        len: usize,
    ) -> std::io::Result<Vec<u8>> {
        // Construct the path to the process's mem file
        let mem_path = format!("/proc/{}/mem", process_info.pid);

        // Open the file in read-only mode
        let mut file = OpenOptions::new().read(true).open(&mem_path)?;

        // Seek to the desired offset
        file.seek(SeekFrom::Start(address))?;

        // Read data into our buffer; handle partial reads if needed
        let mut buf = vec![0u8; len];
        let mut bytes_read = 0;
        while bytes_read < len {
            match file.read(&mut buf[bytes_read..])? {
                0 => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        format!("EOF while reading process memory at address {:#x} in {}", address, mem_path),
                    ));
                }
                n => bytes_read += n,
            }
        }
        Ok(buf)
    }
}

impl IMemoryReader for AndroidMemoryReader {
    /// Reads into a `DynamicStruct` by calling `read_bytes_internal(...)`.
    fn read(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        dynamic_struct: &mut DynamicStruct,
    ) -> bool {
        let size = dynamic_struct.get_size_in_bytes() as usize;

        match self.read_bytes_internal(process_info, address, size) {
            Ok(bytes) => {
                dynamic_struct.copy_from_bytes(&bytes);
                true
            }
            Err(_) => false,
        }
    }

    /// Reads into a raw byte slice by calling `read_bytes_internal(...)`.
    fn read_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &mut [u8],
    ) -> bool {
        let size = values.len();

        match self.read_bytes_internal(process_info, address, size) {
            Ok(bytes) => {
                values.copy_from_slice(&bytes);
                true
            }
            Err(_) => false,
        }
    }
}
