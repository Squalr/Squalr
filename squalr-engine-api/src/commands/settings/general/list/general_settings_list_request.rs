use crate::commands::settings::general::general_settings_command::GeneralSettingsCommand;
use crate::commands::settings::general::general_settings_response::GeneralSettingsResponse;
use crate::commands::settings::general::list::general_settings_list_response::GeneralSettingsListResponse;
use crate::commands::settings::settings_command::SettingsCommand;
use crate::commands::{privileged_command::PrivilegedCommand, privileged_command_request::PrivilegedCommandRequest};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct GeneralSettingsListRequest {}

impl PrivilegedCommandRequest for GeneralSettingsListRequest {
    type ResponseType = GeneralSettingsListResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Settings(SettingsCommand::General {
            general_settings_command: GeneralSettingsCommand::List {
                general_settings_list_request: self.clone(),
            },
        })
    }
}

impl From<GeneralSettingsListResponse> for GeneralSettingsResponse {
    fn from(general_settings_list_response: GeneralSettingsListResponse) -> Self {
        GeneralSettingsResponse::List {
            general_settings_list_response,
        }
    }
}
