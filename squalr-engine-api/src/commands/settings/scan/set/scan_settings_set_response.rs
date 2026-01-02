use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use crate::commands::settings::scan::scan_settings_response::ScanSettingsResponse;
use crate::commands::settings::settings_response::SettingsResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanSettingsSetResponse {}

impl TypedPrivilegedCommandResponse for ScanSettingsSetResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Settings(SettingsResponse::Scan {
            scan_settings_response: ScanSettingsResponse::Set {
                scan_settings_set_response: self.clone(),
            },
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Settings(SettingsResponse::Scan {
            scan_settings_response: ScanSettingsResponse::Set { scan_settings_set_response },
        }) = response
        {
            Ok(scan_settings_set_response)
        } else {
            Err(response)
        }
    }
}
