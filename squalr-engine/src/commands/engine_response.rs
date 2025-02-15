use crate::commands::memory::memory_response::MemoryResponse;
use crate::commands::process::process_response::ProcessResponse;
use crate::commands::project::project_response::ProjectResponse;
use crate::commands::results::results_response::ResultsResponse;
use crate::commands::scan::scan_response::ScanResponse;
use crate::commands::settings::settings_response::SettingsResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineResponse {
    Memory(MemoryResponse),
    Process(ProcessResponse),
    Results(ResultsResponse),
    Project(ProjectResponse),
    Scan(ScanResponse),
    Settings(SettingsResponse),
}

pub trait TypedEngineResponse: Sized {
    fn from_engine_response(response: EngineResponse) -> Result<Self, EngineResponse>;
}

pub trait ExtractArgs {
    type Args: Send;

    fn extract_args(self) -> Self::Args;
}
