use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan_results::set_property::scan_results_set_property_request::ScanResultsSetPropertyRequest;
use squalr_engine_api::commands::scan_results::set_property::scan_results_set_property_response::ScanResultsSetPropertyResponse;
use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ScanResultsSetPropertyRequest {
    type ResponseType = ScanResultsSetPropertyResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        match self.property.get_name() {
            ScanResult::PROPERTY_NAME_VALUE => {
                //
            }
            ScanResult::PROPERTY_NAME_IS_FROZEN => {
                //
            }
            ScanResult::PROPERTY_NAME_ADDRESS | ScanResult::PROPERTY_NAME_MODULE | ScanResult::PROPERTY_NAME_MODULE_OFFSET => {
                log::warn!("Cannot set read-only property {}", self.property.get_name());
            }
            _ => {
                log::warn!("Attempted to set unsupported property on scan result.");
            }
        }

        ScanResultsSetPropertyResponse {}
    }
}
