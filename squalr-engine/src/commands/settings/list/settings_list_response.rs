use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::settings::settings_response::SettingsResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettingsListResponse {}

impl TypedEngineResponse for SettingsListResponse {
    fn from_engine_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Settings(SettingsResponse::List { settings_list_response }) = response {
            Ok(settings_list_response)
        } else {
            Err(response)
        }
    }
}
