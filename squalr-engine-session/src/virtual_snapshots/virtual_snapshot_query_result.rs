use squalr_engine_api::commands::memory::read::memory_read_response::MemoryReadResponse;

#[derive(Clone, Debug, Default)]
pub struct VirtualSnapshotQueryResult {
    pub memory_read_response: Option<MemoryReadResponse>,
    pub resolved_address: Option<u64>,
    pub resolved_module_name: String,
    pub evaluated_pointer_path: String,
}
