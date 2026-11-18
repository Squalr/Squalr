use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::memory::memory_response::MemoryResponse;
use crate::commands::patches::patches_response::PatchesResponse;
use crate::commands::plugins::plugins_response::PluginsResponse;
use crate::commands::pointer_scan::pointer_scan_response::PointerScanResponse;
use crate::commands::process::process_response::ProcessResponse;
use crate::commands::project::project_response::ProjectResponse;
use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::registry::registry_response::RegistryResponse;
use crate::commands::scan::scan_response::ScanResponse;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use crate::commands::settings::settings_response::SettingsResponse;
use crate::commands::struct_scan::struct_scan_response::StructScanResponse;
use crate::commands::trackable_tasks::trackable_tasks_response::TrackableTasksResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PrivilegedCommandResponse {
    Debugger(DebuggerResponse),
    Memory(MemoryResponse),
    Patches(PatchesResponse),
    Plugins(PluginsResponse),
    Process(ProcessResponse),
    Registry(RegistryResponse),
    Results(ScanResultsResponse),
    Project(ProjectResponse),
    ProjectItems(ProjectItemsResponse),
    Scan(ScanResponse),
    PointerScan(PointerScanResponse),
    StructScan(StructScanResponse),
    Settings(SettingsResponse),
    TrackableTasks(TrackableTasksResponse),
    /// The session could not complete the request.
    Failed {
        message: String,
    },
}

impl PrivilegedCommandResponse {
    /// Returns failures common to every adapter, including unsuccessful attachment changes.
    pub fn failure_message(&self) -> Option<&str> {
        match self {
            Self::Failed { message } => Some(message),
            Self::Process(ProcessResponse::Close { process_close_response }) if process_close_response.process_info.is_none() => Some("No process was closed."),
            Self::Process(ProcessResponse::Open { process_open_response }) if process_open_response.opened_process_info.is_none() => {
                Some("No process was opened.")
            }
            Self::Debugger(reply) => reply.failure_message(),
            Self::Memory(reply) => reply.failure_message(),
            Self::Patches(reply) => reply.failure_message(),
            Self::Registry(reply) => reply.failure_message(),
            Self::Results(reply) => reply.failure_message(),
            Self::Project(reply) => reply.failure_message(),
            Self::ProjectItems(reply) => reply.failure_message(),
            Self::Scan(reply) => reply.failure_message(),
            Self::PointerScan(reply) => reply.failure_message(),
            _ => None,
        }
    }
}

pub trait TypedPrivilegedCommandResponse: Sized {
    fn to_engine_response(&self) -> PrivilegedCommandResponse;
    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse>;
}
