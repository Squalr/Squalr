use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::settings::general::general_settings_response::GeneralSettingsResponse;
use crate::commands::settings::settings_response::SettingsResponse;
use crate::structures::settings::general_settings::GeneralSettings;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneralSettingsListResponse {
    pub general_settings: Result<GeneralSettings, String>,
}

impl TypedEngineCommandResponse for GeneralSettingsListResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Settings(SettingsResponse::General {
            general_settings_response: GeneralSettingsResponse::List {
                general_settings_list_response: self.clone(),
            },
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Settings(SettingsResponse::General {
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
