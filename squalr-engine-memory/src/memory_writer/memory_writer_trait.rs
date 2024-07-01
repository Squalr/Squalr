use squalr_engine_common::dynamic_struct::to_bytes::ToBytes;

pub trait IMemoryWriter {
    fn write(&self, process_handle: u64, address: u64, value: &dyn ToBytes);
    fn write_bytes(&self, process_handle: u64, address: u64, values: &[u8]);
}
