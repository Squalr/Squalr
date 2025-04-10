use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::settings::scan::set::scan_settings_set_response::ScanSettingsSetResponse;
use crate::commands::settings::settings_response::SettingsResponse;
use crate::commands::{engine_command_response::EngineCommandResponse, settings::scan::list::scan_settings_list_response::ScanSettingsListResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ScanSettingsResponse {
    Set { scan_settings_set_response: ScanSettingsSetResponse },
    List { scan_settings_list_response: ScanSettingsListResponse },
}

impl TypedEngineCommandResponse for ScanSettingsResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Settings(SettingsResponse::Scan {
            scan_settings_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Settings(SettingsResponse::Scan { scan_settings_response }) = response {
            Ok(scan_settings_response)
        } else {
            Err(response)
        }
    }
}
