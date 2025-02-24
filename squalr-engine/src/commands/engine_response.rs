use crate::commands::memory::memory_response::MemoryResponse;
use crate::commands::process::process_response::ProcessResponse;
use crate::commands::project::project_response::ProjectResponse;
use crate::commands::scan::scan_response::ScanResponse;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use crate::commands::settings::settings_response::SettingsResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineResponse {
    Memory(MemoryResponse),
    Process(ProcessResponse),
    Results(ScanResultsResponse),
    Project(ProjectResponse),
    Scan(ScanResponse),
    Settings(SettingsResponse),
}

pub trait TypedEngineResponse: Sized {
    fn to_engine_response(&self) -> EngineResponse;
    fn from_engine_response(response: EngineResponse) -> Result<Self, EngineResponse>;
}
