use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::settings::general::set::general_settings_set_response::GeneralSettingsSetResponse;
use crate::commands::settings::settings_response::SettingsResponse;
use crate::commands::{engine_command_response::EngineCommandResponse, settings::general::list::general_settings_list_response::GeneralSettingsListResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GeneralSettingsResponse {
    Set {
        general_settings_set_response: GeneralSettingsSetResponse,
    },
    List {
        general_settings_list_response: GeneralSettingsListResponse,
    },
}

impl TypedEngineCommandResponse for GeneralSettingsResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Settings(SettingsResponse::General {
            general_settings_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Settings(SettingsResponse::General { general_settings_response }) = response {
            Ok(general_settings_response)
        } else {
            Err(response)
        }
    }
}
