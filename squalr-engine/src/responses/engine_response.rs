use crate::responses::memory::memory_response::MemoryResponse;
use crate::responses::process::process_response::ProcessResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineResponse {
    Memory(MemoryResponse),
    Process(ProcessResponse),
}
