use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::settings::scan::set::scan_settings_set_request::ScanSettingsSetRequest;
use squalr_engine_api::commands::settings::scan::set::scan_settings_set_response::ScanSettingsSetResponse;
use squalr_engine_scanning::scan_settings_config::ScanSettingsConfig;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ScanSettingsSetRequest {
    type ResponseType = ScanSettingsSetResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        if let Some(results_page_size) = self.results_page_size {
            ScanSettingsConfig::set_results_page_size(results_page_size);
        }

        if let Some(results_read_interval) = self.results_read_interval {
            ScanSettingsConfig::set_results_read_interval(results_read_interval);
        }

        if let Some(project_read_interval) = self.project_read_interval {
            ScanSettingsConfig::set_project_read_interval(project_read_interval);
        }

        if let Some(freeze_interval) = self.freeze_interval {
            ScanSettingsConfig::set_freeze_interval(freeze_interval);
        }

        if let Some(memory_alignment) = self.memory_alignment {
            ScanSettingsConfig::set_memory_alignment(Some(memory_alignment));
        }

        if let Some(floating_point_tolerance) = self.floating_point_tolerance {
            ScanSettingsConfig::set_floating_point_tolerance(floating_point_tolerance);
        }

        ScanSettingsSetResponse {}
    }
}
