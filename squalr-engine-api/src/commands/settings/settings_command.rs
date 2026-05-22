use crate::commands::settings::scan::scan_settings_command::ScanSettingsCommand;
use crate::commands::settings::{general::general_settings_command::GeneralSettingsCommand, memory::memory_settings_command::MemorySettingsCommand};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SettingsCommand {
    General { general_settings_command: GeneralSettingsCommand },
    Memory { memory_settings_command: MemorySettingsCommand },
    Scan { scan_settings_command: ScanSettingsCommand },
}
