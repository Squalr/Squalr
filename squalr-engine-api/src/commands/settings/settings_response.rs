use crate::commands::settings::memory::memory_settings_response::MemorySettingsResponse;
use crate::commands::settings::scan::scan_settings_response::ScanSettingsResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SettingsResponse {
    Memory { memory_settings_response: MemorySettingsResponse },
    Scan { scan_settings_response: ScanSettingsResponse },
}
