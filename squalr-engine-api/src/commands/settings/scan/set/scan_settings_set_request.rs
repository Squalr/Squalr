use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_command_request::EngineCommandRequest;
use crate::commands::settings::scan::scan_settings_command::ScanSettingsCommand;
use crate::commands::settings::scan::scan_settings_response::ScanSettingsResponse;
use crate::commands::settings::scan::set::scan_settings_set_response::ScanSettingsSetResponse;
use crate::commands::settings::settings_command::SettingsCommand;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanSettingsSetRequest {
    #[structopt(name = "setting")]
    pub setting_command: String,
}

impl EngineCommandRequest for ScanSettingsSetRequest {
    type ResponseType = ScanSettingsSetResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Settings(SettingsCommand::Scan {
            scan_settings_command: ScanSettingsCommand::Set {
                scan_settings_set_request: self.clone(),
            },
        })
    }
}

impl From<ScanSettingsSetResponse> for ScanSettingsResponse {
    fn from(scan_settings_set_response: ScanSettingsSetResponse) -> Self {
        ScanSettingsResponse::Set { scan_settings_set_response }
    }
}
