use squalr_engine_common::dynamic_struct::dynamic_struct::DynamicStruct;
use squalr_engine_processes::process_info::OpenedProcessInfo;

pub trait IMemoryReader {
    fn read(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        dynamic_struct: &mut DynamicStruct,
    ) -> bool;
    fn read_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &mut [u8],
    ) -> bool;
}
