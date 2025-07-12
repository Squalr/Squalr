use crate::commands::settings::memory::memory_settings_command::MemorySettingsCommand;
use crate::commands::settings::scan::scan_settings_command::ScanSettingsCommand;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum SettingsCommand {
    Memory {
        #[structopt(flatten)]
        memory_settings_command: MemorySettingsCommand,
    },
    Scan {
        #[structopt(flatten)]
        scan_settings_command: ScanSettingsCommand,
    },
}
