use crate::memory_reader::memory_reader_trait::IMemoryReader;
use squalr_engine_common::dynamic_struct::dynamic_struct::DynamicStruct;
use std::ffi::c_void;
use std::mem;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleA;
use windows_sys::Win32::System::LibraryLoader::GetProcAddress;

type NtReadVirtualMemory = unsafe extern "system" fn(
    process_handle: isize,
    base_address: *const c_void,
    buffer: *mut c_void,
    buffer_size: usize,
    number_of_bytes_read: *mut usize,
) -> i32;

pub struct WindowsMemoryReaderNt {
    nt_read_virtual_memory: NtReadVirtualMemory,
}

impl WindowsMemoryReaderNt {
    // Disable unused compile warning since we ofen swich implementations for testing.
    #[allow(unused)]
    pub fn new() -> Self {
        unsafe {
            let ntdll = GetModuleHandleA("ntdll.dll\0".as_ptr() as *const u8);
            if ntdll == std::ptr::null_mut() {
                panic!("Failed to load ntdll.dll: {}", GetLastError());
            }

            let proc_addr = GetProcAddress(ntdll, "NtReadVirtualMemory\0".as_ptr() as *const u8);
            if proc_addr.is_none() {
                panic!("Failed to locate NtReadVirtualMemory: {}", GetLastError());
            }

            WindowsMemoryReaderNt {
                nt_read_virtual_memory: mem::transmute(proc_addr),
            }
        }
    }
}

impl IMemoryReader for WindowsMemoryReaderNt {
    fn read(
        &self,
        process_handle: u64,
        address: u64,
        dynamic_struct: &mut DynamicStruct,
    ) -> bool {
        let mut buffer = vec![0u8; dynamic_struct.get_size_in_bytes() as usize];

        let success = self.read_bytes(process_handle, address, &mut buffer);
        if success {
            dynamic_struct.copy_from_bytes(&buffer);
        }

        return success;
    }

    fn read_bytes(
        &self,
        process_handle: u64,
        address: u64,
        values: &mut [u8],
    ) -> bool {
        unsafe {
            let size = values.len();
            let mut bytes_read = 0;

            let result = (self.nt_read_virtual_memory)(
                process_handle as isize,
                address as *const c_void,
                values.as_mut_ptr() as *mut c_void,
                size,
                &mut bytes_read,
            );

            return result != 0;
        }
    }
}
