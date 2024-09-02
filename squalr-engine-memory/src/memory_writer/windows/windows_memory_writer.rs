use crate::memory_writer::memory_writer_trait::IMemoryWriter;
use squalr_engine_common::dynamic_struct::to_bytes::ToBytes;
use std::os::raw::c_void;
use std::ptr::null_mut;
use windows_sys::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows_sys::Win32::System::Memory::{VirtualProtectEx, PAGE_READWRITE};

pub struct WindowsMemoryWriter;

impl WindowsMemoryWriter {
    pub fn new() -> Self {
        WindowsMemoryWriter
    }

    fn write_memory(
        process_handle: u64,
        address: u64,
        data: &[u8],
    ) -> bool {
        let mut old_protection = 0;
        let success = unsafe {
            VirtualProtectEx(
                process_handle as *mut c_void,
                address as *mut _,
                data.len(),
                PAGE_READWRITE,
                &mut old_protection,
            );
            let success = WriteProcessMemory(
                process_handle as *mut c_void,
                address as *mut _,
                data.as_ptr() as *const _,
                data.len(),
                null_mut(),
            ) != 0;
            VirtualProtectEx(
                process_handle as *mut c_void,
                address as *mut _,
                data.len(),
                old_protection,
                &mut old_protection,
            );
            success
        };

        return success;
    }
}

impl IMemoryWriter for WindowsMemoryWriter {
    fn write(
        &self,
        process_handle: u64,
        address: u64,
        value: &dyn ToBytes,
    ) {
        let bytes = value.to_bytes();
        Self::write_memory(process_handle, address, &bytes);
    }

    fn write_bytes(
        &self,
        process_handle: u64,
        address: u64,
        values: &[u8],
    ) {
        Self::write_memory(process_handle, address, values);
    }
}
