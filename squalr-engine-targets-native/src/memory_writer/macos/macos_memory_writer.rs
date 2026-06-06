use crate::memory_writer::memory_writer_trait::MemoryWriterTrait;
use mach2::boolean::boolean_t;
use mach2::kern_return::KERN_SUCCESS;
use mach2::message::mach_msg_type_number_t;
use mach2::vm::{mach_vm_protect, mach_vm_region_recurse, mach_vm_write};
use mach2::vm_prot::{VM_PROT_COPY, VM_PROT_READ, VM_PROT_WRITE, vm_prot_t};
use mach2::vm_region::{vm_region_recurse_info_t, vm_region_submap_info_64};
use mach2::vm_types::{mach_vm_address_t, mach_vm_size_t, natural_t, vm_offset_t};
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use std::convert::TryFrom;

pub struct MacOsMemoryWriter;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MacOsWritableRegion {
    base_address: u64,
    region_size: u64,
    original_protection: vm_prot_t,
}

impl MacOsMemoryWriter {
    pub fn new() -> Self {
        MacOsMemoryWriter
    }

    fn write_memory(
        process_handle: u64,
        address: u64,
        data: &[u8],
    ) -> bool {
        if data.is_empty() {
            return true;
        }

        if Self::write_memory_direct(process_handle, address, data) {
            return true;
        }

        Self::write_memory_with_temporary_protection(process_handle, address, data)
    }

    fn write_memory_direct(
        process_handle: u64,
        address: u64,
        data: &[u8],
    ) -> bool {
        let Ok(data_length) = u32::try_from(data.len()) else {
            return false;
        };
        let write_status = unsafe { mach_vm_write(process_handle as _, address as mach_vm_address_t, data.as_ptr() as vm_offset_t, data_length) };

        write_status == KERN_SUCCESS
    }

    fn write_memory_with_temporary_protection(
        process_handle: u64,
        address: u64,
        data: &[u8],
    ) -> bool {
        let Some(end_address) = address.checked_add(data.len() as u64) else {
            return false;
        };
        let writable_regions = Self::query_writable_regions(process_handle, address, end_address);

        if writable_regions.is_empty() {
            return false;
        }

        let mut protected_regions = Vec::new();

        for writable_region in &writable_regions {
            let protected_start_address = writable_region.base_address.max(address);
            let protected_end_address = writable_region
                .base_address
                .saturating_add(writable_region.region_size)
                .min(end_address);

            if protected_start_address >= protected_end_address {
                continue;
            }

            let writable_protection = Self::writable_copy_protection(writable_region.original_protection);
            if Self::protect_memory(
                process_handle,
                protected_start_address,
                protected_end_address - protected_start_address,
                writable_protection,
            ) {
                protected_regions.push(MacOsWritableRegion {
                    base_address: protected_start_address,
                    region_size: protected_end_address - protected_start_address,
                    original_protection: writable_region.original_protection,
                });
            }
        }

        let write_succeeded = protected_regions.len() == writable_regions.len() && Self::write_memory_direct(process_handle, address, data);

        for protected_region in protected_regions.into_iter().rev() {
            if !Self::protect_memory(
                process_handle,
                protected_region.base_address,
                protected_region.region_size,
                protected_region.original_protection,
            ) {
                log::warn!(
                    "Failed to restore macOS memory protection at 0x{:X} after write.",
                    protected_region.base_address
                );
            }
        }

        write_succeeded
    }

    fn query_writable_regions(
        process_handle: u64,
        start_address: u64,
        end_address: u64,
    ) -> Vec<MacOsWritableRegion> {
        let mut writable_regions = Vec::new();
        let mut query_address = start_address as mach_vm_address_t;
        let mut query_depth: natural_t = 0;

        while query_address < end_address as mach_vm_address_t {
            let mut region_base_address = query_address;
            let mut region_size: mach_vm_size_t = 0;
            let mut region_info = vm_region_submap_info_64::default();
            let mut region_info_count: mach_msg_type_number_t = vm_region_submap_info_64::count();
            let query_status = unsafe {
                mach_vm_region_recurse(
                    process_handle as _,
                    &mut region_base_address,
                    &mut region_size,
                    &mut query_depth,
                    (&mut region_info as *mut vm_region_submap_info_64).cast::<i32>() as vm_region_recurse_info_t,
                    &mut region_info_count,
                )
            };

            if query_status != KERN_SUCCESS || region_size == 0 {
                break;
            }

            let region_end_address = region_base_address.saturating_add(region_size);
            if region_end_address > start_address as mach_vm_address_t && region_base_address < end_address as mach_vm_address_t {
                writable_regions.push(MacOsWritableRegion {
                    base_address: region_base_address as u64,
                    region_size: region_size as u64,
                    original_protection: region_info.protection,
                });
            }

            query_address = region_end_address;
        }

        writable_regions
    }

    fn writable_copy_protection(original_protection: vm_prot_t) -> vm_prot_t {
        (original_protection | VM_PROT_READ | VM_PROT_WRITE | VM_PROT_COPY) & !mach2::vm_prot::VM_PROT_EXECUTE
    }

    fn protect_memory(
        process_handle: u64,
        address: u64,
        size: u64,
        protection: vm_prot_t,
    ) -> bool {
        let protect_status = unsafe {
            mach_vm_protect(
                process_handle as _,
                address as mach_vm_address_t,
                size as mach_vm_size_t,
                false as boolean_t,
                protection,
            )
        };

        protect_status == KERN_SUCCESS
    }
}

impl MemoryWriterTrait for MacOsMemoryWriter {
    fn write_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &[u8],
    ) -> bool {
        Self::write_memory(process_info.get_handle(), address, values)
    }
}

#[cfg(test)]
mod tests {
    use super::MacOsMemoryWriter;
    use mach2::kern_return::KERN_SUCCESS;
    use mach2::traps::mach_task_self;
    use mach2::vm::{mach_vm_allocate, mach_vm_deallocate, mach_vm_protect};
    use mach2::vm_prot::{VM_PROT_COPY, VM_PROT_EXECUTE, VM_PROT_READ, VM_PROT_WRITE};
    use mach2::vm_statistics::VM_FLAGS_ANYWHERE;
    use mach2::vm_types::{mach_vm_address_t, mach_vm_size_t};

    struct AllocatedTestRegion {
        task_port: u32,
        address: mach_vm_address_t,
        size: mach_vm_size_t,
    }

    impl Drop for AllocatedTestRegion {
        fn drop(&mut self) {
            unsafe {
                mach_vm_deallocate(self.task_port, self.address, self.size);
            }
        }
    }

    #[test]
    fn writable_copy_protection_adds_write_copy_and_drops_execute() {
        let writable_protection = MacOsMemoryWriter::writable_copy_protection(VM_PROT_READ | VM_PROT_EXECUTE);

        assert_ne!(writable_protection & VM_PROT_READ, 0);
        assert_ne!(writable_protection & VM_PROT_WRITE, 0);
        assert_ne!(writable_protection & VM_PROT_COPY, 0);
        assert_eq!(writable_protection & VM_PROT_EXECUTE, 0);
    }

    #[test]
    fn write_memory_patches_read_execute_region_with_temporary_protection() {
        let task_port = unsafe { mach_task_self() };
        let region_size = 0x1000;
        let mut region_address: mach_vm_address_t = 0;
        let allocate_status = unsafe { mach_vm_allocate(task_port, &mut region_address, region_size, VM_FLAGS_ANYWHERE) };

        assert_eq!(allocate_status, KERN_SUCCESS);

        let allocated_region = AllocatedTestRegion {
            task_port,
            address: region_address,
            size: region_size,
        };
        let initial_bytes = [0xCC_u8, 0xCC, 0xCC, 0xCC];

        unsafe {
            std::ptr::copy_nonoverlapping(initial_bytes.as_ptr(), allocated_region.address as *mut u8, initial_bytes.len());
        }

        let protect_status = unsafe {
            mach_vm_protect(
                allocated_region.task_port,
                allocated_region.address,
                allocated_region.size,
                false as _,
                VM_PROT_READ | VM_PROT_EXECUTE,
            )
        };

        assert_eq!(protect_status, KERN_SUCCESS);

        let patched_bytes = [0x90_u8, 0x90, 0x90, 0x90];
        let write_succeeded = MacOsMemoryWriter::write_memory(allocated_region.task_port as u64, allocated_region.address as u64, &patched_bytes);
        let read_back_bytes = unsafe { std::slice::from_raw_parts(allocated_region.address as *const u8, patched_bytes.len()) };

        assert!(write_succeeded);
        assert_eq!(read_back_bytes, patched_bytes);
    }
}
