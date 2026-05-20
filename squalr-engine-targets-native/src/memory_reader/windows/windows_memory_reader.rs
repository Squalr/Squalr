use crate::memory_reader::memory_reader_trait::MemoryReaderTrait;
use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
use squalr_engine_api::structures::{data_values::data_value::DataValue, processes::opened_process_info::OpenedProcessInfo};
use std::os::raw::c_void;
use windows_sys::Win32::System::Diagnostics::Debug::ReadProcessMemory;

pub struct WindowsMemoryReader;

impl WindowsMemoryReader {
    // Disable unused compile warning since we ofen swich implementations for testing.
    #[allow(unused)]
    pub fn new() -> Self {
        WindowsMemoryReader
    }
}

impl MemoryReaderTrait for WindowsMemoryReader {
    fn read(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        data_value: &mut DataValue,
    ) -> bool {
        unsafe {
            let size = data_value.get_size_in_bytes() as usize;
            let mut buffer = vec![0u8; size];
            let mut bytes_read = 0;

            let result = ReadProcessMemory(
                process_info.get_handle() as *mut c_void,
                address as *const c_void,
                buffer.as_mut_ptr() as *mut c_void,
                size,
                &mut bytes_read,
            );

            data_value.copy_from_bytes(&buffer);

            return result != 0;
        }
    }

    fn read_struct(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        valued_struct: &mut ValuedStruct,
    ) -> bool {
        unsafe {
            let size = valued_struct.get_size_in_bytes() as usize;
            let mut buffer = vec![0u8; size];
            let mut bytes_read = 0;

            let result = ReadProcessMemory(
                process_info.get_handle() as *mut c_void,
                address as *const c_void,
                buffer.as_mut_ptr() as *mut c_void,
                size,
                &mut bytes_read,
            );

            valued_struct.copy_from_bytes(&buffer);

            return result != 0;
        }
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

            let result = ReadProcessMemory(
                process_info.get_handle() as *mut c_void,
                address as *const c_void,
                values.as_mut_ptr() as *mut c_void,
                size,
                &mut bytes_read,
            );

            return result != 0;
        }
    }
}
