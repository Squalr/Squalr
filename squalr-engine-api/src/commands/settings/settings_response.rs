use crate::commands::settings::scan::scan_settings_response::ScanSettingsResponse;
use crate::commands::settings::{general::general_settings_response::GeneralSettingsResponse, memory::memory_settings_response::MemorySettingsResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SettingsResponse {
    General { general_settings_response: GeneralSettingsResponse },
    Memory { memory_settings_response: MemorySettingsResponse },
    Scan { scan_settings_response: ScanSettingsResponse },
}
