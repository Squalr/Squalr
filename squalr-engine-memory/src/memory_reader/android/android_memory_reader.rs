use crate::memory_reader::memory_reader_trait::IMemoryReader;
use squalr_engine_common::dynamic_struct::dynamic_struct::DynamicStruct;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::privileges::android::android_super_user::AndroidSuperUser;
use squalr_engine_processes::process_info::OpenedProcessInfo;
use std::io::{Read, Write};

pub struct AndroidMemoryReader;

impl AndroidMemoryReader {
    pub fn new() -> Self {
        AndroidMemoryReader
    }

    /// Core helper that uses `dd if=/proc/<pid>/mem` to read `len` bytes
    /// from `address` in target process memory, via the *binary shell*.
    fn read_memory_chunk(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        len: usize,
    ) -> std::io::Result<Vec<u8>> {
        Logger::get_instance().log(
            LogLevel::Info,
            &format!("Reading memory (1) from {}: {}, len {}", process_info.pid.as_u32(), address, len),
            None,
        );
        // 1) Acquire the global AndroidSuperUser instance
        let su_instance = AndroidSuperUser::get_instance();
        let mut su = su_instance
            .write()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Failed to lock AndroidSuperUser"))?;

        // 2) Ensure the binary shell is alive; re-spawn if needed
        su.ensure_binary_shell_alive()?;

        // 3) Get a mutable reference to the binary shell process
        let child_bin = su
            .child_binary
            .as_mut()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotConnected, "No binary shell (child_binary) available"))?;

        // 4) Build the dd command
        let dd_command = format!(
            "dd if=/proc/{pid}/mem bs=1 skip={address} count={len} 2>&1",
            pid = process_info.pid.as_u32(),
            address = address,
            len = len
        );

        // 5) Send the command to the shell's stdin
        writeln!(child_bin.child_stdin, "{}", dd_command)?;
        child_bin.child_stdin.flush()?;

        // 6) Read exactly `len` bytes from the shell's stdout
        let stdout_handle = child_bin.child_stdout.get_mut();

        Logger::get_instance().log(
            LogLevel::Info,
            &format!("Awaiting response (2) from {}: {}, len {}", process_info.pid.as_u32(), address, len),
            None,
        );

        let mut buf = vec![0u8; len];
        let mut total_read = 0;
        while total_read < len {
            let read_count = stdout_handle.read(&mut buf[total_read..])?;
            if read_count == 0 {
                return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "EOF while reading memory via dd"));
            }
            total_read += read_count;
        }

        Ok(buf)
    }
}

impl IMemoryReader for AndroidMemoryReader {
    /// Reads into a `DynamicStruct` by calling `read_memory_chunk(...)`.
    fn read(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        dynamic_struct: &mut DynamicStruct,
    ) -> bool {
        let size = dynamic_struct.get_size_in_bytes() as usize;

        match self.read_memory_chunk(process_info, address, size) {
            Ok(bytes) => {
                dynamic_struct.copy_from_bytes(&bytes);
                true
            }
            Err(_) => false,
        }
    }

    /// Reads into a raw byte slice by calling `read_memory_chunk(...)`.
    fn read_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &mut [u8],
    ) -> bool {
        let size = values.len();

        match self.read_memory_chunk(process_info, address, size) {
            Ok(bytes) => {
                values.copy_from_slice(&bytes);
                true
            }
            Err(_) => false,
        }
    }
}
