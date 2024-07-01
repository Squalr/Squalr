extern crate winapi;

use crate::memory_reader::memory_reader_trait::IMemoryReader;
use squalr_engine_common::dynamic_struct::dynamic_struct::DynamicStruct;
use sysinfo::Pid;
use winapi::ctypes::c_void;
use winapi::um::memoryapi::ReadProcessMemory;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winnt::PROCESS_VM_READ;
use winapi::um::handleapi::CloseHandle;

pub struct WindowsMemoryReader;

impl WindowsMemoryReader {
    pub fn new() -> Self {
        WindowsMemoryReader
    }
}

impl IMemoryReader for WindowsMemoryReader {
    fn read(&self, process_id: &Pid, address: u64, dynamic_struct: &mut DynamicStruct) -> Result<(), String> {
        unsafe {
            let handle = OpenProcess(PROCESS_VM_READ, 0, process_id.as_u32());
            if handle.is_null() {
                return Err("Failed to open process".to_string());
            }

            let size = dynamic_struct.size_in_bytes();
            let mut buffer = vec![0u8; size];
            let mut bytes_read = 0;

            let result = ReadProcessMemory(
                handle,
                address as *mut c_void,
                buffer.as_mut_ptr() as *mut c_void,
                size,
                &mut bytes_read,
            );

            if result == 0 {
                return Err("Failed to read process memory".to_string());
            }

            dynamic_struct.copy_from_bytes(&buffer);
            Ok(())
        }
    }

    fn read_bytes(&self, process_id: &Pid, address: u64, values: &mut [u8]) -> Result<(), String> {
        unsafe {
            let handle = OpenProcess(PROCESS_VM_READ, 0, process_id.as_u32());
            if handle.is_null() {
                return Err("Failed to open process".to_string());
            }

            let size = values.len();
            let mut bytes_read = 0;

            let result = ReadProcessMemory(
                handle,
                address as *mut c_void,
                values.as_mut_ptr() as *mut c_void,
                size,
                &mut bytes_read,
            );

            CloseHandle(handle);

            if result == 0 {
                return Err("Failed to read process memory".to_string());
            }

            Ok(())
        }
    }
}
