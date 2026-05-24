use crate::commands::memory::memory_command::MemoryCommand;
use crate::commands::plugins::plugins_command::PluginsCommand;
use crate::commands::pointer_scan::pointer_scan_command::PointerScanCommand;
use crate::commands::process::process_command::ProcessCommand;
use crate::commands::registry::registry_command::RegistryCommand;
use crate::commands::scan::scan_command::ScanCommand;
use crate::commands::scan_results::scan_results_command::ScanResultsCommand;
use crate::commands::settings::settings_command::SettingsCommand;
use crate::commands::struct_scan::struct_scan_command::StructScanCommand;
use crate::commands::trackable_tasks::trackable_tasks_command::TrackableTasksCommand;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PrivilegedCommand {
    Memory(MemoryCommand),
    Plugins(PluginsCommand),
    Process(ProcessCommand),
    Registry(RegistryCommand),
    Results(ScanResultsCommand),
    Scan(ScanCommand),
    PointerScan(PointerScanCommand),
    StructScan(StructScanCommand),
    Settings(SettingsCommand),
    TrackableTasks(TrackableTasksCommand),
}

impl PrivilegedCommand {
    pub fn should_include_privileged_registry_catalog(&self) -> bool {
        matches!(self, Self::Registry(_))
    }
}
