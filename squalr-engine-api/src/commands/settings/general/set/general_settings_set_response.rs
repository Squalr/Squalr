use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::settings::general::general_settings_response::GeneralSettingsResponse;
use crate::commands::settings::settings_response::SettingsResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneralSettingsSetResponse {}

impl TypedEngineCommandResponse for GeneralSettingsSetResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Settings(SettingsResponse::General {
            general_settings_response: GeneralSettingsResponse::Set {
                general_settings_set_response: self.clone(),
            },
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Settings(SettingsResponse::General {
            general_settings_response: GeneralSettingsResponse::Set { general_settings_set_response },
        }) = response
        {
            Ok(general_settings_set_response)
        } else {
            Err(response)
        }
    }
}
