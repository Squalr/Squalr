use crate::memory_reader::memory_reader_trait::MemoryReaderTrait;
use squalr_engine_api::structures::data_values::data_value::DataValue;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
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
                log::error!("Failed to load ntdll.dll, memory reading may fail for this session! Error: {}", GetLastError());
            }

            let proc_addr = GetProcAddress(ntdll, "NtReadVirtualMemory\0".as_ptr() as *const u8);
            if proc_addr.is_none() {
                log::error!(
                    "Failed to locate NtReadVirtualMemory, memory reading may fail for this session! Error: {}",
                    GetLastError()
                );
            }

            WindowsMemoryReaderNt {
                nt_read_virtual_memory: mem::transmute(proc_addr),
            }
        }
    }
}

impl MemoryReaderTrait for WindowsMemoryReaderNt {
    fn read(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        data_value: &mut DataValue,
    ) -> bool {
        let mut buffer = vec![0u8; data_value.get_size_in_bytes() as usize];

        let success = self.read_bytes(process_info, address, &mut buffer);
        if success {
            data_value.copy_from_bytes(&buffer);
        }

        return success;
    }

    fn read_struct(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        valued_struct: &mut ValuedStruct,
    ) -> bool {
        let mut buffer = vec![0u8; valued_struct.get_size_in_bytes() as usize];

        let success = self.read_bytes(process_info, address, &mut buffer);
        if success {
            valued_struct.copy_from_bytes(&buffer);
        }

        return success;
    }

    fn read_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &mut [u8],
    ) -> bool {
        unsafe {
            let size = values.len();
            let mut bytes_read = 0;

            let _unused_result = (self.nt_read_virtual_memory)(
                process_info.get_handle() as isize,
                address as *const c_void,
                values.as_mut_ptr() as *mut c_void,
                size,
                &mut bytes_read,
            );

            return bytes_read == size;
        }
    }
}
