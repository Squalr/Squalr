use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use olorin_engine_api::commands::scan_results::freeze::scan_results_freeze_request::ScanResultsFreezeRequest;
use olorin_engine_api::commands::scan_results::set_property::scan_results_set_property_request::ScanResultsSetPropertyRequest;
use olorin_engine_api::commands::scan_results::set_property::scan_results_set_property_response::ScanResultsSetPropertyResponse;
use olorin_engine_api::structures::scan_results::scan_result::ScanResult;
use olorin_engine_memory::memory_writer::MemoryWriter;
use olorin_engine_memory::memory_writer::memory_writer_trait::IMemoryWriter;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ScanResultsSetPropertyRequest {
    type ResponseType = ScanResultsSetPropertyResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        match self.field_namespace.as_str() {
            ScanResult::PROPERTY_NAME_VALUE => {
                let value_bytes = self.data_value.get_value_bytes();

                for scan_result in &self.scan_results {
                    let address = scan_result.get_address();
                    if let Some(opened_process_info) = engine_privileged_state
                        .get_process_manager()
                        .get_opened_process()
                    {
                        // Best-effort attempt to write the property bytes.
                        let _ = MemoryWriter::get_instance().write_bytes(&opened_process_info, address, &value_bytes);
                    }
                }
            }
            ScanResult::PROPERTY_NAME_IS_FROZEN => {
                // Fire an internal request to freeze.
                let scan_results_freeze_request = ScanResultsFreezeRequest {
                    scan_results: self.scan_results.clone(),
                    is_frozen: false,
                };

                scan_results_freeze_request.execute(engine_privileged_state);
            }
            ScanResult::PROPERTY_NAME_ADDRESS | ScanResult::PROPERTY_NAME_MODULE | ScanResult::PROPERTY_NAME_MODULE_OFFSET => {
                log::warn!("Cannot set read-only property {}", self.field_namespace);
            }
            _ => {
                log::warn!("Attempted to set unsupported property on scan result.");
            }
        }

        ScanResultsSetPropertyResponse {}
    }
}
