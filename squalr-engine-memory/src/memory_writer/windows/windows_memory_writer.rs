extern crate winapi;

use squalr_engine_common::dynamic_struct::to_bytes::ToBytes;
use crate::memory_writer::memory_writer_trait::IMemoryWriter;
use std::ptr::null_mut;
use winapi::ctypes::c_void;
use winapi::um::memoryapi::{WriteProcessMemory, VirtualProtectEx};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winnt::{PROCESS_VM_WRITE, PROCESS_VM_OPERATION, PAGE_READWRITE};
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
        
        if handle.is_null() {
            return false;
        }

        let mut old_protection = 0;
        let success = unsafe {
            VirtualProtectEx(handle, address as *mut c_void, data.len(), PAGE_READWRITE, &mut old_protection);
            let success = WriteProcessMemory(handle, address as *mut c_void, data.as_ptr() as *const c_void, data.len(), null_mut()) != 0;
            VirtualProtectEx(handle, address as *mut c_void, data.len(), old_protection, &mut old_protection);
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
