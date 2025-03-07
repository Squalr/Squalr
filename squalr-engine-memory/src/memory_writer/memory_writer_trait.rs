pub trait IMemoryWriter {
    fn write_bytes(
        &self,
        process_handle: u64,
        address: u64,
        values: &[u8],
    ) -> bool;
}
