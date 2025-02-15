use crate::commands::memory::responses::memory_response::MemoryResponse;
use crate::commands::process::process_response::ProcessResponse;
use crate::commands::scan::scan_response::ScanResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineResponse {
    Memory(MemoryResponse),
    Process(ProcessResponse),
    Scan(ScanResponse),
}

pub trait TypedEngineResponse: Sized {
    fn from_response(response: EngineResponse) -> Result<Self, EngineResponse>;
}

pub trait ExtractArgs {
    type Args: Send;

    fn extract_args(self) -> Self::Args;
}
