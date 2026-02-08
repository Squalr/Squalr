use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use crate::commands::settings::general::general_settings_response::GeneralSettingsResponse;
use crate::commands::settings::settings_error::SettingsError;
use crate::commands::settings::settings_response::SettingsResponse;
use crate::structures::settings::general_settings::GeneralSettings;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneralSettingsListResponse {
    pub general_settings: Result<GeneralSettings, SettingsError>,
}

impl TypedPrivilegedCommandResponse for GeneralSettingsListResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Settings(SettingsResponse::General {
            general_settings_response: GeneralSettingsResponse::List {
                general_settings_list_response: self.clone(),
            },
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Settings(SettingsResponse::General {
            general_settings_response: GeneralSettingsResponse::List {
                general_settings_list_response,
            },
        }) = response
        {
            Ok(general_settings_list_response)
        } else {
            Err(response)
        }
    }
}
