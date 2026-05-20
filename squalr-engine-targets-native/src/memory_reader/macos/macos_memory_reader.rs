use crate::memory_reader::memory_reader_trait::MemoryReaderTrait;
use mach2::kern_return::KERN_SUCCESS;
use mach2::vm::mach_vm_read_overwrite;
use mach2::vm_types::{mach_vm_address_t, mach_vm_size_t};
use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
use squalr_engine_api::structures::{data_values::data_value::DataValue, processes::opened_process_info::OpenedProcessInfo};

pub struct MacOsMemoryReader;

impl MacOsMemoryReader {
    pub fn new() -> Self {
        MacOsMemoryReader
    }

    fn read_process_memory(
        process_handle: u64,
        source_address: u64,
        destination_buffer: &mut [u8],
    ) -> bool {
        if destination_buffer.is_empty() {
            return true;
        }

        let mut copied_bytes: mach_vm_size_t = 0;
        let read_status = unsafe {
            mach_vm_read_overwrite(
                process_handle as _,
                source_address as mach_vm_address_t,
                destination_buffer.len() as mach_vm_size_t,
                destination_buffer.as_mut_ptr() as mach_vm_address_t,
                &mut copied_bytes as *mut mach_vm_size_t,
            )
        };

        read_status == KERN_SUCCESS && copied_bytes as usize == destination_buffer.len()
    }
}

impl MemoryReaderTrait for MacOsMemoryReader {
    fn read(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        data_value: &mut DataValue,
    ) -> bool {
        let mut value_bytes = vec![0u8; data_value.get_size_in_bytes() as usize];
        let read_succeeded = Self::read_process_memory(process_info.get_handle(), address, &mut value_bytes);

        if read_succeeded {
            data_value.copy_from_bytes(&value_bytes);
        }

        read_succeeded
    }

    fn read_struct(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        valued_struct: &mut ValuedStruct,
    ) -> bool {
        let mut struct_bytes = vec![0u8; valued_struct.get_size_in_bytes() as usize];
        let read_succeeded = Self::read_process_memory(process_info.get_handle(), address, &mut struct_bytes);

        if read_succeeded {
            return valued_struct.copy_from_bytes(&struct_bytes);
        }

        false
    }

    fn read_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &mut [u8],
    ) -> bool {
        Self::read_process_memory(process_info.get_handle(), address, values)
    }
}
