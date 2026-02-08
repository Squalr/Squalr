use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use crate::commands::settings::scan::scan_settings_command::ScanSettingsCommand;
use crate::commands::settings::scan::scan_settings_response::ScanSettingsResponse;
use crate::commands::settings::scan::set::scan_settings_set_response::ScanSettingsSetResponse;
use crate::commands::settings::settings_command::SettingsCommand;
use crate::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use crate::structures::scanning::memory_read_mode::MemoryReadMode;
use crate::{commands::privileged_command::PrivilegedCommand, structures::memory::memory_alignment::MemoryAlignment};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Default, Serialize, Deserialize)]
pub struct ScanSettingsSetRequest {
    #[structopt(long)]
    pub results_page_size: Option<u32>,
    #[structopt(long)]
    pub results_read_interval_ms: Option<u64>,
    #[structopt(long)]
    pub project_read_interval_ms: Option<u64>,
    #[structopt(long)]
    pub freeze_interval_ms: Option<u64>,
    #[structopt(long)]
    pub memory_alignment: Option<MemoryAlignment>,
    #[structopt(long)]
    pub memory_read_mode: Option<MemoryReadMode>,
    #[structopt(long)]
    pub floating_point_tolerance: Option<FloatingPointTolerance>,
    #[structopt(long)]
    pub is_single_threaded_scan: Option<bool>,
    #[structopt(long)]
    pub debug_perform_validation_scan: Option<bool>,
}

impl PrivilegedCommandRequest for ScanSettingsSetRequest {
    type ResponseType = ScanSettingsSetResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Settings(SettingsCommand::Scan {
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
