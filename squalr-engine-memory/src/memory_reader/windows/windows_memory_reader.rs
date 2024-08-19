use crate::memory_reader::memory_reader_trait::IMemoryReader;
use squalr_engine_common::dynamic_struct::dynamic_struct::DynamicStruct;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use std::os::raw::c_void;

pub struct WindowsMemoryReader;

impl WindowsMemoryReader {
    #[allow(unused)] // Disable unused compile warning for now
    pub fn new() -> Self {
        WindowsMemoryReader
    }
}

impl IMemoryReader for WindowsMemoryReader {
    fn read(&self, process_handle: u64, address: u64, dynamic_struct: &mut DynamicStruct) -> Result<(), String> {
        unsafe {
            let size = dynamic_struct.size_in_bytes() as usize;
            let mut buffer = vec![0u8; size];
            let mut bytes_read = 0;

            let result = ReadProcessMemory(
                process_handle as isize,
                address as *const c_void,
                buffer.as_mut_ptr() as *mut c_void,
                size,
                &mut bytes_read,
            );

            if result == 0 {
                return Err(format!("Failed to read process memory: {}", GetLastError()));
            }

            dynamic_struct.copy_from_bytes(&buffer);
            return Ok(());
        }
    }

    fn read_bytes(&self, process_handle: u64, address: u64, values: &mut [u8]) -> Result<(), String> {
        unsafe {
            let size = values.len();
            let mut bytes_read = 0;

            let result = ReadProcessMemory(
                process_handle as isize,
                address as *const c_void,
                values.as_mut_ptr() as *mut c_void,
                size,
                &mut bytes_read,
            );

            if result == 0 {
                return Err(format!("Failed to read process memory: {}", GetLastError()));
            }

            return Ok(());
        }
    }
}
