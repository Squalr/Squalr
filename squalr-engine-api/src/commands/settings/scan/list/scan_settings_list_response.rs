use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use crate::commands::settings::scan::scan_settings_response::ScanSettingsResponse;
use crate::commands::settings::settings_error::SettingsError;
use crate::commands::settings::settings_response::SettingsResponse;
use crate::structures::settings::scan_settings::ScanSettings;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanSettingsListResponse {
    pub scan_settings: Result<ScanSettings, SettingsError>,
}

impl TypedPrivilegedCommandResponse for ScanSettingsListResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Settings(SettingsResponse::Scan {
            scan_settings_response: ScanSettingsResponse::List {
                scan_settings_list_response: self.clone(),
            },
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Settings(SettingsResponse::Scan {
            scan_settings_response: ScanSettingsResponse::List { scan_settings_list_response },
        }) = response
        {
            Ok(scan_settings_list_response)
        } else {
            Err(response)
        }
    }
}
