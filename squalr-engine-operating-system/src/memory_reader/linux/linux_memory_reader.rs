use crate::memory_reader::memory_reader_trait::MemoryReaderTrait;
use libc::{c_void, iovec, pid_t, process_vm_readv};
use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
use squalr_engine_api::structures::{data_values::data_value::DataValue, processes::opened_process_info::OpenedProcessInfo};

pub struct LinuxMemoryReader;

impl LinuxMemoryReader {
    pub fn new() -> Self {
        LinuxMemoryReader
    }

    fn read_process_memory(
        process_id: u32,
        source_address: u64,
        destination_buffer: &mut [u8],
    ) -> bool {
        if destination_buffer.is_empty() {
            return true;
        }

        let local_iovec = iovec {
            iov_base: destination_buffer.as_mut_ptr() as *mut c_void,
            iov_len: destination_buffer.len(),
        };

        let remote_iovec = iovec {
            iov_base: source_address as *mut c_void,
            iov_len: destination_buffer.len(),
        };

        let bytes_read = unsafe { process_vm_readv(process_id as pid_t, &local_iovec, 1, &remote_iovec, 1, 0) };

        bytes_read == destination_buffer.len() as isize
    }
}

impl MemoryReaderTrait for LinuxMemoryReader {
    fn read(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        data_value: &mut DataValue,
    ) -> bool {
        let mut value_bytes = vec![0u8; data_value.get_size_in_bytes() as usize];
        let read_succeeded = Self::read_process_memory(process_info.get_process_id_raw(), address, &mut value_bytes);

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
        let read_succeeded = Self::read_process_memory(process_info.get_process_id_raw(), address, &mut struct_bytes);

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
        Self::read_process_memory(process_info.get_process_id_raw(), address, values)
    }
}
