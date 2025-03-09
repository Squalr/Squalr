use crate::commands::settings::list::settings_list_response::SettingsListResponse;
use crate::commands::settings::settings_command::SettingsCommand;
use crate::commands::settings::settings_response::SettingsResponse;
use crate::commands::{engine_command::EngineCommand, engine_command_request::EngineCommandRequest};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct SettingsListRequest {
    #[structopt(short = "s", long)]
    pub scan: bool,
    #[structopt(short = "m", long)]
    pub memory: bool,
    #[structopt(short = "a", long)]
    pub list_all: bool,
}

impl EngineCommandRequest for SettingsListRequest {
    type ResponseType = SettingsListResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Settings(SettingsCommand::List {
            settings_list_request: self.clone(),
        })
    }
}

impl From<SettingsListResponse> for SettingsResponse {
    fn from(settings_list_response: SettingsListResponse) -> Self {
        SettingsResponse::List { settings_list_response }
    }
}
