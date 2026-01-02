use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use crate::commands::settings::general::general_settings_response::GeneralSettingsResponse;
use crate::commands::settings::settings_response::SettingsResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneralSettingsSetResponse {}

impl TypedPrivilegedCommandResponse for GeneralSettingsSetResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Settings(SettingsResponse::General {
            general_settings_response: GeneralSettingsResponse::Set {
                general_settings_set_response: self.clone(),
            },
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Settings(SettingsResponse::General {
            general_settings_response: GeneralSettingsResponse::Set { general_settings_set_response },
        }) = response
        {
            Ok(general_settings_set_response)
        } else {
            Err(response)
        }
    }
}
