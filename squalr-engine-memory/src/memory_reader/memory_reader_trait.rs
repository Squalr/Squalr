use sysinfo::Pid;
use squalr_engine_common::dynamic_struct::dynamic_struct::DynamicStruct;

pub trait IMemoryReader {
    fn read(&self, process_handle: u64, address: u64, dynamic_struct: &mut DynamicStruct) -> Result<(), String>;
    fn read_bytes(&self, process_handle: u64, address: u64, values: &mut [u8]) -> Result<(), String>;
}
