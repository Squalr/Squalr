use crate::memory_reader::memory_reader_trait::IMemoryReader;
use squalr_engine_common::dynamic_struct::dynamic_struct::DynamicStruct;
use std::ffi::c_void;
use std::mem;
use std::ptr::null_mut;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::RtlNtStatusToDosError;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::NTSTATUS;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleA;
use windows_sys::Win32::System::LibraryLoader::GetProcAddress;
use windows_sys::Win32::System::Memory::MEMORY_MAPPED_VIEW_ADDRESS;
use windows_sys::Win32::System::Memory::PAGE_READONLY;
use windows_sys::Win32::System::Memory::SECTION_ALL_ACCESS;
use windows_sys::Win32::System::Memory::SEC_COMMIT;

type NtCreateSection = unsafe extern "system" fn(
    section_handle: *mut HANDLE,
    desired_access: u32,
    object_attributes: *mut c_void,
    maximum_size: *mut u64,
    section_page_protection: u32,
    allocation_attributes: u32,
    file_handle: HANDLE,
) -> NTSTATUS;

type NtMapViewOfSection = unsafe extern "system" fn(
    section_handle: HANDLE,
    process_handle: HANDLE,
    base_address: *mut *mut c_void,
    zero_bits: usize,
    commit_size: usize,
    section_offset: *mut u64,
    view_size: *mut u64,
    inherit_disposition: u32,
    allocation_type: u32,
    win32_protect: u32,
) -> NTSTATUS;

pub struct WindowsMemoryReaderNtMapped {
    nt_create_section: NtCreateSection,
    nt_map_view_of_section: NtMapViewOfSection,
}

// Shit doesn't work.
impl WindowsMemoryReaderNtMapped {
    #[allow(unused)] // Disable unused compile warning for now
    pub fn new() -> Self {
        unsafe {
            let ntdll = GetModuleHandleA("ntdll.dll\0".as_ptr() as *const u8);
            if ntdll == 0 {
                panic!("Failed to load ntdll.dll: {}", GetLastError());
            }

            let create_section_addr = GetProcAddress(ntdll, "NtCreateSection\0".as_ptr() as *const u8);
            if create_section_addr.is_none() {
                panic!("Failed to locate NtCreateSection: {}", GetLastError());
            }

            let map_view_addr = GetProcAddress(ntdll, "NtMapViewOfSection\0".as_ptr() as *const u8);
            if map_view_addr.is_none() {
                panic!("Failed to locate NtMapViewOfSection: {}", GetLastError());
            }

            WindowsMemoryReaderNtMapped {
                nt_create_section: mem::transmute(create_section_addr),
                nt_map_view_of_section: mem::transmute(map_view_addr),
            }
        }
    }

    unsafe fn map_memory(
        &self,
        process_handle: HANDLE,
        address: u64,
        size: usize,
    ) -> Option<*mut c_void> {
        let mut section_handle: HANDLE = 0;
        let mut view_size = size as u64;

        let status = (self.nt_create_section)(
            &mut section_handle,
            SECTION_ALL_ACCESS,
            null_mut(),
            &mut view_size,
            PAGE_READONLY,
            SEC_COMMIT,
            0 as HANDLE,
        );

        if status != 0 {
            let error_code = RtlNtStatusToDosError(status);
            panic!("NtCreateSection failed with error code: {:#x} (NTSTATUS: {:#x})", error_code, status);
        }

        let mut out_mapped_address: *mut c_void = null_mut();

        let status = (self.nt_map_view_of_section)(
            section_handle,
            process_handle,
            &mut out_mapped_address,
            0,
            size,
            &mut (address as u64),
            &mut view_size,
            2, // ViewUnmap
            0, // AllocationType
            PAGE_READONLY,
        );

        if status != 0 {
            let error_code = RtlNtStatusToDosError(status);
            panic!("NtMapViewOfSection (local) failed with error code: {:#x} (NTSTATUS: {:#x})", error_code, status);
        }

        return Some(out_mapped_address);
    }

    unsafe fn unmap_memory(
        &self,
        mapped_address: *mut c_void,
    ) {
        let unmapping = MEMORY_MAPPED_VIEW_ADDRESS { Value: mapped_address };
        windows_sys::Win32::System::Memory::UnmapViewOfFile(unmapping);
    }
}

impl IMemoryReader for WindowsMemoryReaderNtMapped {
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

        success
    }

    fn read_bytes(
        &self,
        process_handle: u64,
        address: u64,
        values: &mut [u8],
    ) -> bool {
        unsafe {
            let size = values.len();
            if let Some(mapped_address) = self.map_memory(process_handle as HANDLE, address, size) {
                let buffer = std::slice::from_raw_parts(mapped_address as *const u8, size);
                values.copy_from_slice(buffer);
                self.unmap_memory(mapped_address);
                return true;
            }

            false
        }
    }
}
