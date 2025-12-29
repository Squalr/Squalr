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

        if let Some(results_read_interval_ms) = self.results_read_interval_ms {
            ScanSettingsConfig::set_results_read_interval_ms(results_read_interval_ms);
        }

        if let Some(project_read_interval_ms) = self.project_read_interval_ms {
            ScanSettingsConfig::set_project_read_interval_ms(project_read_interval_ms);
        }

        if let Some(freeze_interval_ms) = self.freeze_interval_ms {
            ScanSettingsConfig::set_freeze_interval_ms(freeze_interval_ms);
        }

        if let Some(memory_alignment) = self.memory_alignment {
            ScanSettingsConfig::set_memory_alignment(Some(memory_alignment));
        }

        if let Some(memory_read_mode) = self.memory_read_mode {
            ScanSettingsConfig::set_memory_read_mode(memory_read_mode);
        }

        if let Some(floating_point_tolerance) = self.floating_point_tolerance {
            ScanSettingsConfig::set_floating_point_tolerance(floating_point_tolerance);
        }

        if let Some(is_single_threaded_scan) = self.is_single_threaded_scan {
            ScanSettingsConfig::set_is_single_threaded_scan(is_single_threaded_scan);
        }

        if let Some(debug_perform_validation_scan) = self.debug_perform_validation_scan {
            ScanSettingsConfig::set_debug_perform_validation_scan(debug_perform_validation_scan);
        }

        ScanSettingsSetResponse {}
    }
}
