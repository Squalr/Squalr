use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_command_request::EngineCommandRequest;
use crate::commands::settings::general::general_settings_command::GeneralSettingsCommand;
use crate::commands::settings::general::general_settings_response::GeneralSettingsResponse;
use crate::commands::settings::general::set::general_settings_set_response::GeneralSettingsSetResponse;
use crate::commands::settings::settings_command::SettingsCommand;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Default, Serialize, Deserialize)]
pub struct GeneralSettingsSetRequest {
    #[structopt(short = "r_delay", long)]
    pub engine_request_delay: Option<u64>,
}

impl EngineCommandRequest for GeneralSettingsSetRequest {
    type ResponseType = GeneralSettingsSetResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Settings(SettingsCommand::General {
            general_settings_command: GeneralSettingsCommand::Set {
                general_settings_set_request: self.clone(),
            },
        })
    }
}

impl From<GeneralSettingsSetResponse> for GeneralSettingsResponse {
    fn from(general_settings_set_response: GeneralSettingsSetResponse) -> Self {
        GeneralSettingsResponse::Set { general_settings_set_response }
    }
}
