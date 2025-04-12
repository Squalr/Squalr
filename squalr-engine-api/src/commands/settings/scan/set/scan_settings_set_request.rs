use crate::commands::engine_command_request::EngineCommandRequest;
use crate::commands::settings::scan::scan_settings_command::ScanSettingsCommand;
use crate::commands::settings::scan::scan_settings_response::ScanSettingsResponse;
use crate::commands::settings::scan::set::scan_settings_set_response::ScanSettingsSetResponse;
use crate::commands::settings::settings_command::SettingsCommand;
use crate::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use crate::{commands::engine_command::EngineCommand, structures::memory::memory_alignment::MemoryAlignment};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Default, Serialize, Deserialize)]
pub struct ScanSettingsSetRequest {
    #[structopt(short = "psize", long)]
    pub results_page_size: Option<u32>,
    #[structopt(short = "r_read_interval", long)]
    pub results_read_interval: Option<u32>,
    #[structopt(short = "p_read_interval", long)]
    pub project_read_interval: Option<u32>,
    #[structopt(short = "f_interval", long)]
    pub freeze_interval: Option<u32>,
    #[structopt(short = "m_align", long)]
    pub memory_alignment: Option<MemoryAlignment>,
    #[structopt(short = "f_tol", long)]
    pub floating_point_tolerance: Option<FloatingPointTolerance>,
    #[structopt(short = "st", long)]
    pub is_single_threaded_scan: bool,
    #[structopt(short = "dbg", long)]
    pub debug_perform_validation_scan: bool,
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
