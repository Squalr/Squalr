use crate::commands::engine_request::EngineRequest;
use crate::commands::settings::settings_command::SettingsCommand;
use crate::commands::settings::settings_response::SettingsResponse;
use crate::commands::{engine_command::EngineCommand, settings::set::settings_set_response::SettingsSetResponse};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct SettingsSetRequest {
    #[structopt(name = "setting")]
    pub setting_command: String,
}

impl EngineRequest for SettingsSetRequest {
    type ResponseType = SettingsSetResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Settings(SettingsCommand::Set {
            settings_set_request: self.clone(),
        })
    }
}

impl From<SettingsSetResponse> for SettingsResponse {
    fn from(settings_set_response: SettingsSetResponse) -> Self {
        SettingsResponse::Set { settings_set_response }
    }
}
