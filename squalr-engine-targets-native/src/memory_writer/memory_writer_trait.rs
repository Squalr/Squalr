use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;

pub trait MemoryWriterTrait {
    fn write_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &[u8],
    ) -> bool;
}
