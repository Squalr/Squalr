use crate::memory_writer::memory_writer_trait::IMemoryWriter;

use windows_sys::Win32::System::Threading::OpenProcess;
use windows_sys::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows_sys::Win32::System::Threading::{PROCESS_VM_WRITE, PROCESS_VM_OPERATION};
use windows_sys::Win32::System::Memory::{VirtualProtectEx, PAGE_READWRITE};
use windows_sys::Win32::Foundation::CloseHandle;
use squalr_engine_common::dynamic_struct::to_bytes::ToBytes;
use std::ptr::null_mut;
use sysinfo::Pid;

pub struct WindowsMemoryWriter;

impl WindowsMemoryWriter {
    pub fn new() -> Self {
        WindowsMemoryWriter
    }
    
    fn write_memory(process_id: &Pid, address: u64, data: &[u8]) -> bool {
        let handle = unsafe {
            OpenProcess(PROCESS_VM_WRITE | PROCESS_VM_OPERATION, 0, process_id.as_u32())
        };
        
        if handle <= 0{
            return false;
        }

        let mut old_protection = 0;
        let success = unsafe {
            VirtualProtectEx(handle, address as *mut _, data.len(), PAGE_READWRITE, &mut old_protection);
            let success = WriteProcessMemory(handle, address as *mut _, data.as_ptr() as *const _, data.len(), null_mut()) != 0;
            VirtualProtectEx(handle, address as *mut _, data.len(), old_protection, &mut old_protection);
            CloseHandle(handle);
            success
        };

        return success;
    }
}

impl IMemoryWriter for WindowsMemoryWriter {
    fn write(&self, process_id: &Pid, address: u64, value: &dyn ToBytes) {
        let bytes = value.to_bytes();
        Self::write_memory(process_id, address, &bytes);
    }

    fn write_bytes(&self, process_id: &Pid, address: u64, values: &[u8]) {
        Self::write_memory(process_id, address, values);
    }
}
