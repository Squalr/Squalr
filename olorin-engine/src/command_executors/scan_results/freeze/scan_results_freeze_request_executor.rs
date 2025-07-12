use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use olorin_engine_api::commands::scan_results::freeze::scan_results_freeze_request::ScanResultsFreezeRequest;
use olorin_engine_api::commands::scan_results::freeze::scan_results_freeze_response::ScanResultsFreezeResponse;
use olorin_engine_memory::memory_reader::MemoryReader;
use olorin_engine_memory::memory_reader::memory_reader_trait::IMemoryReader;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ScanResultsFreezeRequest {
    type ResponseType = ScanResultsFreezeResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        if let Ok(snapshot_scan_result_freeze_list) = engine_privileged_state
            .get_snapshot_scan_result_freeze_list()
            .read()
        {
            for scan_result in &self.scan_results {
                let address = scan_result.get_address();
                if self.is_frozen {
                    if let Some(opened_process_info) = engine_privileged_state
                        .get_process_manager()
                        .get_opened_process()
                    {
                        if let Some(mut data_value) = scan_result.get_data_type().get_default_value() {
                            if MemoryReader::get_instance().read(&opened_process_info, address, &mut data_value) {
                                snapshot_scan_result_freeze_list.set_address_frozen(address, data_value);
                            }
                        }
                    }
                } else {
                    snapshot_scan_result_freeze_list.set_address_unfrozen(address);
                }
            }
        }
        ScanResultsFreezeResponse {}
    }
}
