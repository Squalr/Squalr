use crate::commands::memory::memory_command::MemoryCommand;
use crate::commands::pointer_scan::pointer_scan_command::PointerScanCommand;
use crate::commands::process::process_command::ProcessCommand;
use crate::commands::scan::scan_command::ScanCommand;
use crate::commands::scan_results::scan_results_command::ScanResultsCommand;
use crate::commands::settings::settings_command::SettingsCommand;
use crate::commands::struct_scan::struct_scan_command::StructScanCommand;
use crate::commands::trackable_tasks::trackable_tasks_command::TrackableTasksCommand;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum PrivilegedCommand {
    #[structopt(alias = "mem", alias = "m")]
    Memory(MemoryCommand),

    #[structopt(alias = "proc", alias = "pr")]
    Process(ProcessCommand),

    #[structopt(alias = "res", alias = "r")]
    Results(ScanResultsCommand),

    #[structopt(alias = "scan", alias = "s")]
    Scan(ScanCommand),

    #[structopt(alias = "pscan")]
    PointerScan(PointerScanCommand),

    #[structopt(alias = "sscan")]
    StructScan(StructScanCommand),

    #[structopt(alias = "set", alias = "st")]
    Settings(SettingsCommand),

    #[structopt(alias = "tasks", alias = "tt")]
    TrackableTasks(TrackableTasksCommand),
}
