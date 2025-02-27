use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::settings::settings_response::SettingsResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettingsSetResponse {}

impl TypedEngineResponse for SettingsSetResponse {
    fn to_engine_response(&self) -> EngineResponse {
        EngineResponse::Settings(SettingsResponse::Set {
            settings_set_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Settings(SettingsResponse::Set { settings_set_response }) = response {
            Ok(settings_set_response)
        } else {
            Err(response)
        }
    }
}
