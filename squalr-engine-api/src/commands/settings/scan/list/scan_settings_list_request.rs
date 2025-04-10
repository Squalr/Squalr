use crate::commands::settings::scan::list::scan_settings_list_response::ScanSettingsListResponse;
use crate::commands::settings::scan::scan_settings_command::ScanSettingsCommand;
use crate::commands::settings::scan::scan_settings_response::ScanSettingsResponse;
use crate::commands::settings::settings_command::SettingsCommand;
use crate::commands::{engine_command::EngineCommand, engine_command_request::EngineCommandRequest};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanSettingsListRequest {}

impl EngineCommandRequest for ScanSettingsListRequest {
    type ResponseType = ScanSettingsListResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Settings(SettingsCommand::Scan {
            scan_settings_command: ScanSettingsCommand::List {
                scan_settings_list_request: self.clone(),
            },
        })
    }
}

impl From<ScanSettingsListResponse> for ScanSettingsResponse {
    fn from(scan_settings_list_response: ScanSettingsListResponse) -> Self {
        ScanSettingsResponse::List { scan_settings_list_response }
    }
}
