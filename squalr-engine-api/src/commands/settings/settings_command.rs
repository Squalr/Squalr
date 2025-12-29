use crate::commands::settings::scan::scan_settings_command::ScanSettingsCommand;
use crate::commands::settings::{general::general_settings_command::GeneralSettingsCommand, memory::memory_settings_command::MemorySettingsCommand};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum SettingsCommand {
    General {
        #[structopt(flatten)]
        general_settings_command: GeneralSettingsCommand,
    },
    Memory {
        #[structopt(flatten)]
        memory_settings_command: MemorySettingsCommand,
    },
    Scan {
        #[structopt(flatten)]
        scan_settings_command: ScanSettingsCommand,
    },
}
